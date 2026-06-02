# xray-app 实现计划

## 依赖模块接口

### xray-model (路径: src/model/src/types.rs)

### xray-services (路径: src/services/src/types.rs)

### xray-ui (路径: src/ui/src/types.rs)

## 本模块错误类型 (types.rs 已生成)

```rust
pub enum AppError {
    InitError,
    FatalError,
}
```

## 实现列表

- [x] **终端生命周期管理**
  - [x] 启用 Raw mode 与进入 Alternate screen
  - [x] 终端退出时的自动还原
- [x] **状态初始化**
  - [x] 从 `Storage` 加载持久化状态
  - [x] 创建 `XrayService` 和 `SystemdService` 实例
- [x] **主循环逻辑**
  - [x] 事件轮询与分发 (16ms tick)
  - [x] 定时状态刷新 (每 30 ticks)
- [x] **资源保存**
  - [x] 正常退出时保存 `AppState`
