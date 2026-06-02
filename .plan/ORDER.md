# 执行顺序 — xray-tui

## 阶段 1

- **xray-model** (src/model) — Xray 配置完整数据模型。定义所有 serde 结构体、枚举和多态序列化逻辑。

- **xray-services** (src/services) — 核心服务层。Xray 配置读写、systemd 管理、acme.sh 证书管理、订阅链接生成、
journalctl 日志采集、状态持久化。

  依赖: xray-model
- **xray-ui** (src/ui) — 终端用户界面层。所有屏幕渲染、事件处理、可复用 UI widget。

  依赖: xray-model, xray-services
- **xray-app** (src) — 应用入口。终端初始化、服务实例创建、主事件循环。

  依赖: xray-model, xray-services, xray-ui

