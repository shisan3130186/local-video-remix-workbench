# 本地短视频批量混剪工作台

## 项目定位

这是一个本地桌面端视频批量处理软件，目标是实现：
导入素材 → 视频预览 → 视频切片 → 随机混剪 → 批量导出。

## 当前阶段

当前版本：V0.3.4-dev
当前状态：本地混剪主链路已可用
已完成：导入、预览、切片、随机抽取、拼接、批量生成、画布适配、基础效果、画中画、封面帧
下一步：分类混剪

## 技术栈

桌面端：
Tauri / Vue3 / TypeScript

视频处理：
Rust / FFmpeg / FFprobe

数据库：
SQLite（后续用于保存素材、任务、导出记录和配置）

AI能力：
后续接入 TTS / ASR / OCR / 文案匹配。

## 启动方式

开发启动：

```powershell
cd E:\Workspace\Project_03_本地短视频批量混剪工作台
corepack pnpm tauri dev
```

前端构建检查：

```powershell
cd E:\Workspace\Project_03_本地短视频批量混剪工作台
corepack pnpm build
```

Rust / Tauri 后端检查：

```powershell
cd E:\Workspace\Project_03_本地短视频批量混剪工作台\src-tauri
$env:CARGO_HTTP_CHECK_REVOKE='false'
cargo check
```

本地打包：

```powershell
cd E:\Workspace\Project_03_本地短视频批量混剪工作台
corepack pnpm tauri build
```

## 当前开发规则

开发前必须阅读：
- AGENTS.md
- PROJECT_STATUS.md
- ROADMAP.md
- TODO_NEXT.md
- DECISIONS.md
