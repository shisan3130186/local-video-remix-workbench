# CURRENT_TASK.md

> 最后更新：2026-07-17。只记录当前唯一任务、真实中断点和下一动作。

## 当前唯一任务

等待用户确认是否开始第一阶段第3项：素材库持久化第一版。

## 已确认范围

1. 输出格式第一版固定MP4，视频H.264、音频AAC。
2. 分辨率支持跟随画布、720p和1080p。
3. 帧率支持跟随源视频、24、25、30、50和60fps。
4. 质量支持省空间、标准和高清三个档位，用清晰易懂的选项代替直接填写复杂码率。
5. 编码支持自动、CPU、NVIDIA、Intel和AMD。
6. GPU可用性由Rust调用FFmpeg做极小实际编码测试，不只检查编码器名称。
7. 用户选择的编码器只作用于最终成片；平滑片段和配音分镜等临时文件继续使用CPU稳定编码。
8. 基础导出、随机混剪、分类混剪、批量生成、AI无配音和AI配音必须统一生效。

## 不在本次范围

- MOV、MKV、WebM等其他封装格式。
- H.265、AV1和自定义编码器参数。
- 用户手填任意宽高、任意码率或专业编码预设。
- 同时开发AI音色库、水印、ASR或安装包。

## 当前状态

状态：完整输出设置第一版已经通过用户人工验收，本轮提交本地Git，不推GitHub；下一项开始前等待用户确认。

已完成：

- 新增统一输出设置模型与旧项目默认值兼容。
- 新增CPU、NVIDIA、Intel、AMD实际编码检测和自动推荐。
- 基础导出与所有最终混剪统一应用分辨率、帧率、质量和编码器。
- 新输出设置自动进入项目保存与恢复。
- 输出设置页完成可视化状态、不可用GPU禁选和重新检测。
- 前端正式构建、Rust格式、编译、严格Clippy和87项测试通过。
- 真实FFmpeg测试通过：CPU与NVIDIA可用；横竖屏720p和25fps正确；质量档位文件大小递增。
- 用户人工验收通过：右侧入口、输出选项和实际导出均可正常使用。

## 预计修改文件

前端：

- `src/services/videoMixService.ts`
- `src/composables/useRemixSettings.ts`
- `src/components/tool-settings/ExportSettingsPanel.vue`
- `src/components/tool-settings/types.ts`
- `src/components/ToolSettingModal.vue`
- `src/App.vue`
- 基础导出、混剪和TTS相关Service
- 项目恢复兼容逻辑和输出设置样式

Rust：

- 新增 `src-tauri/src/video_engine/output.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/video_engine/render.rs`
- `src-tauri/src/video_engine/mix.rs`
- `src-tauri/src/video_engine/mod.rs`

## 关键风险

1. GPU编码器即使出现在FFmpeg列表中，也可能因显卡或驱动不可用而启动失败。
2. 720p/1080p需要正确适配横屏、竖屏、方屏和原画，不得拉伸画面。
3. 旧项目快照没有新输出字段，恢复时必须安全回退到推荐默认值。
4. NVIDIA、Intel、AMD和CPU的参数不同，必须在Rust统一封装，禁止前端拼命令。

## 验收标准

1. 输出设置页能看懂当前格式、分辨率、帧率、质量和编码方式。
2. GPU检测显示检测中、可用、不可用及简单原因。
3. 720p与1080p导出尺寸正确，横竖屏不变形。
4. 选择固定帧率后，成片帧率与设置一致。
5. 三个质量档位生成的文件大小有合理差异。
6. 选择可用GPU后能成功导出；不可用GPU不能被误选。
7. 所有最终成片链路统一生效，原有字幕、配音、BGM、画布和任务取消不被破坏。
8. 自动检查和用户人工验收通过后才提交本地Git，不推GitHub。

## 下一动作

1. 用户打开“导出设置”检查界面和GPU检测结果。
2. 用户分别测试720p/1080p、固定帧率、质量档位和可用GPU导出。
3. 用户验收通过后提交本地Git，不推GitHub。
