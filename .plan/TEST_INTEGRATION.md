# 集成测试计划

## 跨模块测试

2. xray-services + xray-model 联调
   在真实环境中测试 xray x25519、journalctl 调用

3. xray-ui + xray-model, xray-services 联调
   在终端中手动测试所有导航路径和表单交互

4. xray-app + xray-model, xray-services, xray-ui 联调
   端到端测试：启动应用、创建配置、导出链接、退出

