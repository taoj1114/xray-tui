# xray-model 实现计划

## 本模块错误类型 (types.rs 已生成)

```rust
pub enum ModelError {
    SerdeError,
    ValidationError,
}
```

## 实现列表

- [x] **基础配置模型** (config.rs)
  - [x] `XrayConfig` 顶级结构体
  - [x] `InboundConfig` / `OutboundConfig`
  - [x] 协议支持: VMess, VLESS, Trojan, Shadowsocks, Http, Socks
  - [x] 传输层: TCP, WebSocket, gRPC, HttpUpgrade, Quic, KCP
  - [x] 安全层: TLS, REALITY
- [x] **路由模型** (routing.rs)
  - [x] `RoutingConfig` 结构体
  - [x] `RoutingRule` 及其多态序列化
  - [x] 预设规则 (Bypass CN, Ads, etc.)
- [x] **证书与状态模型** (ssl.rs, settings.rs)
  - [x] `CertInfo` 结构体
  - [x] `GlobalSettings` 配置
  - [x] `AppState` 全局持久化状态
