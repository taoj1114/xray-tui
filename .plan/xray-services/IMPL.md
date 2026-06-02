# xray-services 实现计划

## 依赖模块接口

### xray-model (路径: src/model/src/types.rs)

## 本模块错误类型 (types.rs 已生成)

```rust
pub enum ServiceError {
    IoError,
    JsonError,
    SystemdError,
    AcmeError,
    StorageError,
    SubscriptionError,
    JournalError,
}
```

## 实现列表

- [x] **XrayService** (xray.rs)
  - [x] 配置生成与写入
  - [x] x25519 密钥对生成
  - [x] Xray 自动安装与卸载脚本
- [x] **SystemdService** (systemd.rs)
  - [x] 单元文件生成与安装
  - [x] 服务状态获取 (解析 /proc 获取 CPU/内存)
  - [x] 服务启动/停止/重启
- [x] **AcmeService** (acme.sh wrapper)
  - [x] acme.sh 自动安装
  - [x] 证书申请 (Webroot, ALPN, DNS Cloudflare)
  - [x] 证书列表与续期
- [x] **SubscriptionService** (subscription.rs)
  - [x] 分享链接生成 (VMess, VLESS, Trojan, SS)
  - [x] 订阅内容 Base64 编码导出
- [x] **JournalService** (journal.rs)
  - [x] 通过 journalctl 获取实时日志
  - [x] 级别与关键词过滤
- [x] **Storage** (storage.rs)
  - [x] JSON 格式状态持久化
