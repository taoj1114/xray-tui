# xray-ui 测试计划

## 单元测试
每个屏幕的 handle_key 和 render 逻辑通过模拟事件测试

## 集成测试
在终端中手动测试所有导航路径和表单交互
## 验证命令

- 单元测试: `cargo test -p xray-ui`
- 集成测试: `cargo test -p xray-ui --test integration`
