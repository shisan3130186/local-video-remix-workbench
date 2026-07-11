# TASK

## 2026-07-11：修复 AI HTTPS 连接失败

状态：已修复且自动检查通过，等待用户重新启动验证真实方舟请求。

问题：Windows 无法完成证书吊销状态查询，导致 Rust AI 请求在 HTTPS 握手阶段失败。

处理：AI 客户端使用内置可信根证书完成证书和域名校验，不依赖 Windows 在线吊销查询。

## 2026-07-11：AI 文案智能混剪 MVP

状态：开发和自动检查已完成，等待真实火山引擎环境人工验收。

本次目标：
1. 切片后准备片段编号、真实时长和预览图。
2. Rust 后端读取环境变量并调用火山引擎方舟多模态模型。
3. 严格校验 `orderedSegmentIds`。
4. 前端展示排序结果并支持上移、下移、删除。
5. 复用现有 `VideoEngine` 混剪导出链路生成视频。

本次不做：
1. 字幕烧录。
2. TTS。
3. 会员系统。
4. 复杂时间轴。
5. 完整视频上传。
6. AI 自动发布。

验收命令：
```text
corepack pnpm build
cargo fmt
cargo check
```

提交信息：
```text
feat(ai): add script based remix planning
```
