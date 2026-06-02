# xray-services 测试计划

## 单元测试
Mock D-Bus 和文件系统，测试各服务逻辑分支

## 集成测试
在真实环境中测试 xray x25519、journalctl 调用
## 验证命令

- 单元测试: `cargo test -p xray-services`
- 集成测试: `cargo test -p xray-services --test integration`
