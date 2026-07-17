# CODEX_START_HERE.md

> 用途：这是项目的低 Token 启动入口。每次继续开发先读本文件和 `CURRENT_TASK.md`，不要默认整份读取历史文档。

## 1. 项目一句话目标

开发一款 Windows 本地桌面端 AI 短视频批量混剪工作台，完成素材导入、分析、切片、分镜、配音、字幕和批量导出，并最终打包给普通用户测试。

## 2. 固定技术栈

- 桌面端：Tauri 2 + Vue 3 + TypeScript。
- 本地视频处理：Rust + FFmpeg / FFprobe。
- 数据库：SQLite，素材库阶段再正式接入。
- AI/TTS：优先云端API，密钥使用Windows当前用户加密保存。
- 包管理：pnpm + cargo。

未经用户确认，不更换技术栈，不重写整个项目。

## 3. 绝对红线

1. 页面和前端Service禁止直接拼FFmpeg命令，所有视频处理必须走Rust VideoEngine。
2. 禁止明文保存API Key、密码、凭证或会员密钥。
3. 会员状态不能只靠本地判断，商业版必须远程验证。
4. 一次只开发一个可人工验收的功能，不同时铺开多个大模块。
5. 不删除或破坏已经人工验收的功能。
6. 不引入大型依赖，除非先说明必要性并获得用户确认。
7. 测试素材放 `E:\Workspace\测试素材库`。
8. 测试结果放 `E:\Workspace\测试结果库`。
9. 新问题按规则记录到 `E:\Workspace\01_全局复利踩坑日志.md`。
10. 用户人工验收通过后才提交本地Git；暂不推送GitHub。

## 4. 当前已经稳定可用

- 多视频/文件夹导入、预览、固定切片、智能场景切片和多帧理解。
- 随机混剪、分类混剪、一次生成1～10条AI差异视频。
- 画布、模糊背景、镜像、旋转、变速、画面调整、画中画和本地BGM。
- AI画面理解、本地文案断句、逐句分镜和人工调整。
- 火山TTS逐句配音、可选原声、自然短句字幕。
- AI/TTS密钥Windows加密保存。
- AI/TTS临时失败最多3次受控重试。
- 最近项目自动保存和启动恢复。
- 任务取消、FFmpeg真实进度、失败条目单独重试和临时文件清理。
- 完整输出设置，包括分辨率、帧率、质量档位和GPU编码。
- SQLite素材库、AI分析缓存、失效素材重新定位和手动载入。
- 智能切片、多帧理解、视频内容提炼和AI自动镜头分类。
- 最近一次完整基线：内容提炼与AI自动分类已通过人工验收，Rust 96项测试通过。

最新已验收本地提交：

- `c1dd651 feat: 支持项目自动保存与恢复`
- `2db9154 feat: 增加AI和TTS失败自动重试`
- `6de4e28 feat(config): 支持软件内安全保存AI和TTS密钥`
- `94b6b80 feat: 完成完整输出设置和GPU编码选择`

## 5. 当前开发顺序

1. 已完成并验收：第一阶段稳定生产基础，包括任务控制、完整输出设置和SQLite素材库。
2. 已完成并验收：智能切片与多帧理解第一版。
3. 已完成并验收：视频内容提炼和AI自动分类。
4. 下一候选：音频输入、云端ASR和句子时间轴。
5. 可视化AI音色库和水印工具。
6. 内置FFmpeg、首次启动向导和用户测试包。
7. 商业阶段再做会员、用户中心、代码签名和自动更新。

当前唯一任务与中断点必须以 `CURRENT_TASK.md` 为准。

## 6. 核心代码地图

- 页面编排：`src/App.vue`
- 素材导入与切片：`src/features/materials`
- AI分镜与AI成片：`src/features/ai-remix`
- 普通混剪与导出：`src/features/remix-export`
- TTS与配音成片：`src/features/tts`
- 项目恢复：`src/features/project-recovery`
- 底部任务栏：`src/components/TaskControlBar.vue`
- Tauri命令入口：`src-tauri/src/lib.rs`
- VideoEngine：`src-tauri/src/video_engine`
- AI后端：`src-tauri/src/ai_remix.rs`
- TTS后端：`src-tauri/src/tts.rs`

## 7. 固定验证命令

前端：

```powershell
corepack pnpm build
```

Rust：

```powershell
cd src-tauri
cargo fmt -- --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

人工验收：双击项目根目录 `启动验收.bat`。

## 8. 详细文档路由

不要全文读取，先用关键词定位：

- 架构分层：`docs/02_技术架构说明书.md`
- FFmpeg/VideoEngine：`docs/05_视频处理引擎设计.md`
- 历史完成记录：`PROJECT_STATUS.md`
- 历史技术决定：`DECISIONS.md`
- 路线规划：`ROADMAP.md`
- 当前候选任务：`TODO_NEXT.md`
- 安装包审计：`docs/12_代码质量与安装包可行性审计.md`

## 9. 每次开发的最短流程

1. 读本文件、`CURRENT_TASK.md` 和 `git status`。
2. 用 `rg` 找本次相关代码，只读命中函数附近内容。
3. 先说明目标、范围、文件和验证方式；用户已确认范围时直接继续。
4. 使用 `apply_patch` 修改文件。
5. 做与风险相称的自动检查。
6. 更新 `CURRENT_TASK.md`；阶段完成时再同步历史文档。
7. 交给用户人工验收；通过后提交本地Git，不推GitHub。
