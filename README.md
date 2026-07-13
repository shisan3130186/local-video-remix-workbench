# 本地短视频批量混剪工作台

## 项目定位

这是一个本地桌面端视频批量处理软件，目标是实现：
导入素材 → 视频预览 → 视频切片 → 随机混剪 → 批量导出。

## 当前阶段

当前代码版本：V0.3.4
当前状态：AI文案智能混剪MVP已完成人工验收；前端结构整理前三个阶段均已完成人工回归。
已完成：多视频导入、全部素材切片、片段预览图和时长、AI 画面理解、文案逐句分镜、主画面和候选画面、人工调整、VideoEngine 导出、画布、平滑混剪、视频效果、画中画、本地 BGM 和导出结果列表。
下一步：先规划随机混剪、分类混剪、批量生成和基础导出模块整理；安装包和新增功能继续暂缓。

## 技术栈

桌面端：
Tauri / Vue3 / TypeScript

视频处理：
Rust / FFmpeg / FFprobe

数据库：
SQLite（后续用于保存素材、任务、导出记录和配置）

AI 能力：
当前通过火山引擎方舟完成预览图画面理解和文案逐句分镜匹配。TTS、ASR、OCR 和自动字幕尚未实现。

AI 配置只从启动软件的 PowerShell 环境变量读取：

```powershell
$env:AI_API_KEY="你的 API Key"
$env:AI_BASE_URL="https://ark.cn-beijing.volces.com/api/v3"
$env:AI_MODEL="你的模型接入点 ID"
```

API Key 不进入前端、不写日志，也不能提交到 Git。使用 `$env:` 配置时，必须在同一个 PowerShell 窗口启动软件。

## 启动方式

最简单方式：

双击项目根目录中的 `启动验收.bat`。

说明：
这个脚本内部使用英文输出，是为了避免 Windows `cmd` 把中文批处理内容解析成乱码命令。

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

## Windows 安装包状态

NSIS 安装器已经可以成功生成，但当前只适合内部技术测试：安装包尚未内置 FFmpeg / FFprobe，AI 仍需要从启动环境读取配置，安装包也没有代码签名。

详细结论见 `docs/12_代码质量与安装包可行性审计.md`，发布前检查见 `tasks/安装包发布准备清单.md`。

## 当前开发规则

开发前必须阅读：
- AGENTS.md
- PROJECT_STATUS.md
- ROADMAP.md
- TODO_NEXT.md
- DECISIONS.md
