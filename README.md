# 本地短视频批量混剪工作台

## 项目定位

这是一个本地桌面端视频批量处理软件，目标是实现：
导入素材 → 视频预览 → 视频切片 → 随机混剪 → 批量导出。

## 当前阶段

当前代码版本：V0.3.4
当前状态：AI文案智能混剪主链路已支持一次生成1～10条差异视频、软件内API密钥安全保存、AI/TTS临时失败自动重试和最近项目自动恢复。
已完成：多视频导入、全部素材切片、AI画面理解、文案分镜、1～10条差异方案、无配音顺序批量导出、TTS一次生成多版本复用、AI/TTS最多3次受控重试、最近项目保存与恢复、Windows加密密钥保存、画布、平滑混剪、视频效果、画中画、本地BGM、原声混合、自动字幕和导出结果列表。
下一步：用户人工验收项目保存与恢复，验收通过后再确定普通用户安装包或音频输入方向。

多条差异视频规则：
1. 默认生成3条，可设置1～10条。
2. 第一条始终保留用户当前确认的分镜。
3. 后续版本只轮换各句备选画面，不改变文案和分镜顺序。
4. 每条视频内部不会重复使用同一个片段，重复方案会自动跳过。
5. 备选不足时只生成实际可组成的不重复版本。
6. 视频按顺序生成；某一条失败后会记录原因并继续。
7. AI配音只逐句生成一次，后续视频复用同一批配音，避免重复付费。

AI/TTS自动重试规则：
1. 网络超时、临时限流和常见服务故障最多尝试3次。
2. 任务日志会显示等待时间和当前尝试次数。
3. Key、权限、模型配置或明确额度耗尽直接报错，不盲目重试。
4. 已成功的片段描述和配音句子会保留，只补失败部分。

项目保存与恢复规则：
1. 有素材、文案或输出目录后，软件会自动保存最近一个项目。
2. 顶部显示自动保存状态和最近保存时间。
3. 软件重新打开时可以恢复或放弃上次记录。
4. 素材、BGM、画中画或输出目录失效时会明确提示并安全降级。
5. 项目快照不包含API Key，密钥继续单独加密保存。

## 技术栈

桌面端：
Tauri / Vue3 / TypeScript

视频处理：
Rust / FFmpeg / FFprobe

数据库：
SQLite（后续用于保存素材、任务、导出记录和配置）

AI 能力：
当前通过火山引擎方舟完成预览图画面理解和文案逐句分镜匹配，并通过火山语音完成文案生成MP3、本地试听和逐句配音视频。逐句配音支持可选保留完整原声、调节音量和自动生成自然短句字幕；ASR、OCR、逐字字幕和人声/BGM分离尚未实现。

推荐配置方式：打开工作台右侧“API 密钥”，填写方舟API Key、模型接入点ID和TTS API Key，然后点击“保存并立即使用”。

安全规则：
1. 密钥通过Windows当前用户的数据保护机制加密后保存。
2. 配置文件不是明文，复制到其他Windows账户后无法直接解密。
3. 软件只向前端返回“是否已配置”等状态，不会把已保存的完整密钥重新回显。
4. 密钥不会进入任务日志、Git或普通项目配置文件。
5. 密钥输入框留空保存时，会保留已有密钥；可以在设置页分别删除AI或TTS密钥。

环境变量继续作为开发备用，软件内保存的配置优先：

```powershell
$env:AI_API_KEY="你的 API Key"
$env:AI_BASE_URL="https://ark.cn-beijing.volces.com/api/v3"
$env:AI_MODEL="你的模型接入点 ID"
```

使用环境变量时，必须在同一个PowerShell窗口启动软件。环境变量不会写入本机加密配置。

TTS第一阶段使用火山引擎豆包语音的独立API Key，与方舟 `AI_API_KEY` 不是同一个Key：

```powershell
$env:TTS_API_KEY="你的火山语音 API Key"
$env:TTS_RESOURCE_ID="seed-tts-2.0"
$env:TTS_SPEAKER="zh_female_vv_uranus_bigtts"
```

`TTS_RESOURCE_ID` 和 `TTS_SPEAKER` 有上述默认值，普通用户通常只需填写TTS API Key。密钥提交保存后由Rust加密处理，不进入任务日志。

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
