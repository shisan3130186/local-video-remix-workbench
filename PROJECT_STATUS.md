# PROJECT_STATUS.md

## 当前版本

v0.1.0

## 项目定位

本地桌面端 AI 视频批量混剪工作台。

## 当前阶段

阶段一：基础视频导入、预览、FFmpeg处理引擎打通。

当前核心目标：

```text
FFmpeg 检测 → 导入素材 → 读取信息 → 视频预览 → 基础导出 → 任务日志
```

## 已完成

- [x] 首页 UI
- [x] 视频导入按钮
- [x] 基础页面布局
- [x] 项目整体开发框架规划
- [x] FFmpeg环境检测
- [ ] 视频信息读取
- [ ] 视频预览
- [ ] 基础导出任务

## 当前阻塞

Tauri 启动依赖 Rust / Cargo。当前 Rustup 已安装，但 stable 工具链安装不完整，缺少 rustc，导致 `corepack pnpm tauri dev` 无法启动。

## 当前禁止修改

- 不允许更换 Tauri
- 不允许重做首页
- 不允许新增会员系统
- 不允许开发 AI 内容理解
- 不允许开发 API Key 系统
- 不允许开发 TTS
- 不允许开发 ASR 字幕识别
- 不允许进行 UI 大重构

## 最近一次开发内容

日期：2026-06-11
完成：完成项目启动基础检查；前端依赖已安装，Vue / TypeScript / Vite 构建通过，FFmpeg / FFprobe 命令可返回版本；Tauri 启动阻塞在 Rust 工具链不完整。
涉及文件：
- PROJECT_STATUS.md
- src-tauri/tauri.conf.json
- pnpm-lock.yaml
- pnpm-workspace.yaml
遗留问题：需要修复 Rust stable 工具链，确保 cargo 和 rustc 可用后再启动 Tauri。

## 下一步建议

1. 完成视频信息读取
2. 完成单视频导入
3. 完成导入视频后的预览
