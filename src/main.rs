use axum::{routing::post, Router, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use async_trait::async_trait;
use anyhow::anyhow;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use reqwest::Client;
use tracing::error;

// 事件定义
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Event {
    id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    timestamp: DateTime<Utc>,
    channels: Vec<String>,
}

// 通道适配器 trait
#[async_trait]
trait ChannelAdapter: Send + Sync {
    fn name(&self) -> String;
    async fn send(&self, event: &Event) -> anyhow::Result<()>;
}

// Webhook 适配器
struct WebhookAdapter {
    url: String,
    client: Client,
}

#[async_trait]
impl ChannelAdapter for WebhookAdapter {
    fn name(&self) -> String {
        "webhook".to_string()
    }

    async fn send(&self, event: &Event) -> anyhow::Result<()> {
        self.client
            .post(&self.url)
            .json(event)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

// Kafka 适配器
struct KafkaAdapter {
    producer: FutureProducer,
    topic: String,
}

#[async_trait]
impl ChannelAdapter for KafkaAdapter {
    fn name(&self) -> String {
        "kafka".to_string()
    }

    async fn send(&self, event: &Event) -> anyhow::Result<()> {
        let payload = serde_json::to_string(event)?;
        let event_id_str = event.id.to_string();
        let record = FutureRecord::to(&self.topic)
            .payload(&payload)
            .key(&event_id_str);
        self.producer
            .send(record, Timeout::Never)
            .await
            .map_err(|(e, _)| anyhow!(e))?;
        Ok(())
    }
}

// MQTT 适配器
struct MqttAdapter {
    client: AsyncClient,
    topic: String,
}

#[async_trait]
impl ChannelAdapter for MqttAdapter {
    fn name(&self) -> String {
        "mqtt".to_string()
    }

    async fn send(&self, event: &Event) -> anyhow::Result<()> {
        let payload = serde_json::to_string(event)?;
        self.client
            .publish(&self.topic, QoS::AtLeastOnce, false, payload)
            .await?;
        Ok(())
    }
}

// 事件生产者
async fn handle_event(
    Json(event): Json<Event>,
    tx: mpsc::Sender<Event>,
) -> Result<(), axum::http::StatusCode> {
    tx.send(event).await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn start_producer(tx: mpsc::Sender<Event>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/event", post(|event| handle_event(event, tx)));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// 事件总线
async fn event_bus(mut rx: mpsc::Receiver<Event>, adapters: Vec<Box<dyn ChannelAdapter>>) {
    while let Some(event) = rx.recv().await {
        for adapter in &adapters {
            if event.channels.contains(&adapter.name()) {
                if let Err(e) = adapter.send(&event).await {
                    error!("Failed to send event to {}: {}", adapter.name(), e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建事件通道
    let (tx, rx) = mpsc::channel::<Event>(100);

    // 初始化通道适配器
    let webhook = WebhookAdapter {
        url: "http://localhost:8080/webhook".to_string(),
        client: Client::new(),
    };

    // let kafka_producer = rdkafka::producer::FutureProducer::from_config(
    //     &rdkafka::config::ClientConfig::new()
    //         .set("bootstrap.servers", "localhost:9092")
    //         .create()?
    // )?;
    let kafka_producer = rdkafka::config::ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()?;

    let kafka = KafkaAdapter {
        producer: kafka_producer,
        topic: "events".to_string(),
    };

    let mqtt_options = MqttOptions::new("rust-client", "localhost", 1883);
    let (mqtt_client, mut event_loop) = AsyncClient::new(mqtt_options, 10);
    tokio::spawn(async move { while event_loop.poll().await.is_ok() {} });
    let mqtt = MqttAdapter {
        client: mqtt_client,
        topic: "events".to_string(),
    };

    let adapters: Vec<Box<dyn ChannelAdapter>> = vec![
        Box::new(webhook),
        Box::new(kafka),
        Box::new(mqtt),
    ];

    // 启动事件总线
    let bus_handle = tokio::spawn(event_bus(rx, adapters));

    // 启动生产者
    start_producer(tx).await?;

    bus_handle.await?;
    Ok(())
}