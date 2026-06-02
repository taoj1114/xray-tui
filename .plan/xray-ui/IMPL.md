# xray-ui 实现计划

## 依赖模块接口

### xray-model (路径: src/model/src/types.rs)

### xray-services (路径: src/services/src/types.rs)

## 本模块错误类型 (types.rs 已生成)

```rust
pub enum UiError {
    RenderError,
    InputError,
}
```

## 实现列表

- [x] **框架基础**
  - [x] `App` 状态机与 `Action` 模式
  - [x] 多层级屏幕栈 (Screen Stack)
  - [x] 全局快捷键处理 (Tab 切换, Esc 返回)
- [x] **主要屏幕**
  - [x] `Dashboard`: 状态概览与服务控制
  - [x] `InboundList`: 入站配置管理
  - [x] `InboundWizard`: 交互式配置创建向导
  - [x] `UserManager`: 协议用户增删改查
  - [x] `RoutingEditor`: 路由规则可视化编辑
  - [x] `SslManagement`: Acme 证书自动化管理
  - [x] `LogViewer`: 实时日志查看
  - [x] `Settings`: 应用全局参数设置
- [x] **交互组件**
  - [x] `ConfirmDialog`: 二次确认弹窗
  - [x] `ShareExport`: 链接展示与导出界面
