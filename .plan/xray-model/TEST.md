# xray-model 测试计划

## 单元测试
每种协议×传输×安全组合序列化为 JSON，与 Xray 官方文档示例对比

## 集成测试
无外部依赖，仅单元测试
## 验证命令

- 单元测试: `cargo test -p xray-model`
- 集成测试: `cargo test -p xray-model --test integration`
