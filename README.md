# 事件通知系统
该项目仅用作Rust语言学习与记录，项目代码可能存在错误或不规范，欢迎批评指正。<br/>
文章地址：<br/>
[第一部分]https://mp.weixin.qq.com/s/zlui9eiqrncAkzP-9AGZ5w<br/>
[第二部分]https://mp.weixin.qq.com/s/nIgmYYDvCX65t31DtzVOTw<br/>
[第三部分]https://mp.weixin.qq.com/s/jrqGSxh-dKLcSo92Zd9uog<br/>
## 项目概述
`event_notification_system` 是一个用 Rust 语言编写的事件通知系统，借助多个高性能库实现了异步处理、序列化、消息队列集成、HTTP 请求等功能。该系统能用于处理和分发各类事件通知，具备高并发、可扩展的特性。

## 项目依赖
以下是项目 `Cargo.toml` 里的主要依赖：
- **`tokio` (1.40)**: 异步运行时，为项目提供异步 I/O 支持，实现高并发处理。
- **`serde` (1.0) 与 `serde_json` (1.0)**: 用于数据的序列化和反序列化，方便在不同数据格式间转换。
- **`uuid` (1.10)**: 生成通用唯一识别码，可用于事件或消息的唯一标识。
- **`chrono` (0.4)**: 处理日期和时间，在事件记录和定时任务中可能会用到。
- **`rdkafka` (0.36)**: 集成 Kafka 消息队列，实现事件的发布和订阅。
- **`rumqttc` (0.24)**: 支持 MQTT 协议，可用于物联网设备的事件通知。
- **`reqwest` (0.12)**: 发起 HTTP 请求，可用于向外部服务发送事件通知。
- **`tracing` (0.1) 与 `tracing-subscriber` (0.3)**: 日志追踪和记录，方便调试和监控项目运行状态。
- **`thiserror` (1.0) 与 `anyhow` (1.0)**: 错误处理库，简化错误处理逻辑。
- **`crossbeam-channel` (0.5)**: 提供多生产者多消费者的通道，用于线程间通信。
- **`openssl` (0.10)**: 提供 SSL/TLS 加密功能，保障数据传输安全。
- **`axum` (0.8.4)**: 构建高性能的 HTTP 服务器，可能用于接收事件请求。
- **`async-trait` (0.1)**: 支持异步特征，方便编写异步代码。

