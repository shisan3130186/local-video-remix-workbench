# 本地短视频批量混剪工作台

## 项目定位

这是一个本地桌面端视频批量处理软件，目标是实现：
导入素材 → 视频预览 → 视频切片 → 随机混剪 → 批量导出。

## 当前阶段

当前代码版本：V0.3.4
当前状态：AI文案智能混剪主链路已支持一次生成1～10条差异视频；自动检查已通过，真实AI/TTS多素材回归仍待配置密钥和补充测试素材。
已完成：多视频导入、全部素材切片、片段预览图和时长、AI画面理解、本地自然短句分镜、AI固定短句画面匹配、人工调整、1～10条差异方案、无配音顺序批量导出、TTS一次生成多版本复用、单条失败继续、画布、平滑混剪、视频效果、画中画、本地BGM、原声混合、自动字幕和导出结果列表。
下一步：使用至少3组真实素材完成AI/TTS稳定性回归；随后按差距报告继续选择音频输入驱动、素材库持久化或项目保存与失败恢复。

多条差异视频规则：
1. 默认生成3条，可设置1～10条。
2. 第一条始终保留用户当前确认的分镜。
3. 后续版本只轮换各句备选画面，不改变文案和分镜顺序。
4. 每条视频内部不会重复使用同一个片段，重复方案会自动跳过。
5. 备选不足时只生成实际可组成的不重复版本。
6. 视频按顺序生成；某一条失败后会记录原因并继续。
7. AI配音只逐句生成一次，后续视频复用同一批配音，避免重复付费。

## 技术栈

桌面端：
Tauri / Vue3 / TypeScript

视频处理：
Rust / FFmpeg / FFprobe

数据库：
SQLite（后续用于保存素材、任务、导出记录和配置）

AI 能力：
当前通过火山引擎方舟完成预览图画面理解和文案逐句分镜匹配，并通过火山语音完成文案生成MP3、本地试听和逐句配音视频。逐句配音支持可选保留完整原声、调节音量和自动生成自然短句字幕；ASR、OCR、逐字字幕和人声/BGM分离尚未实现。

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
