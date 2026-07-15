# 本地短视频批量混剪工作台

## 项目定位

这是一个本地桌面端视频批量处理软件，目标是实现：
导入素材 → 视频预览 → 视频切片 → 随机混剪 → 批量导出。

## 当前阶段

当前代码版本：V0.3.4
当前状态：AI文案智能混剪MVP、前五个前端重构阶段、TTS第一阶段和TTS第二阶段均已通过人工验收。
已完成：多视频导入、全部素材切片、片段预览图和时长、AI画面理解、文案逐句分镜、人工调整、VideoEngine导出、画布、平滑混剪、视频效果、画中画、本地BGM、导出结果列表、文案生成MP3试听、逐句配音与画面时长同步，以及可选原声混合。
下一步：等待用户确认新的单功能阶段，推荐优先规划自动字幕第一阶段；当前配音规则为原声默认关闭、打开后默认15%、尾帧冻结或画面裁短、固定1.0x、单音色、不生成字幕。

## 技术栈

桌面端：
Tauri / Vue3 / TypeScript

视频处理：
Rust / FFmpeg / FFprobe

数据库：
SQLite（后续用于保存素材、任务、导出记录和配置）

AI 能力：
当前通过火山引擎方舟完成预览图画面理解和文案逐句分镜匹配，并通过火山语音完成文案生成MP3、本地试听和逐句配音视频。逐句配音支持可选保留完整原声并调节音量；ASR、OCR、自动字幕和人声/BGM分离尚未实现。

AI 配置只从启动软件的 PowerShell 环境变量读取：

```powershell
$env:AI_API_KEY="你的 API Key"
$env:AI_BASE_URL="https://ark.cn-beijing.volces.com/api/v3"
$env:AI_MODEL="你的模型接入点 ID"
```

API Key 不进入前端、不写日志，也不能提交到 Git。使用 `$env:` 配置时，必须在同一个 PowerShell 窗口启动软件。

TTS第一阶段使用火山引擎豆包语音的独立API Key，与方舟 `AI_API_KEY` 不是同一个Key：

```powershell
$env:TTS_API_KEY="你的火山语音 API Key"
$env:TTS_RESOURCE_ID="seed-tts-2.0"
$env:TTS_SPEAKER="zh_female_vv_uranus_bigtts"
```

`TTS_RESOURCE_ID` 和 `TTS_SPEAKER` 有上述默认值，内部验收至少需要配置 `TTS_API_KEY`。TTS Key同样只由Rust读取，不进入前端和日志。

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
