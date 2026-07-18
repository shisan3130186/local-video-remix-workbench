# CURRENT_TASK.md

> 最后更新：2026-07-18。只记录当前任务、真实中断点和下一动作。

## 当前唯一任务

开发“可视化AI音色库第一版”。用户已确认继续，要求参考竞品的筛选和卡片结构，增加语种并优先选择自然音色。人工验收前不提交Git，不推GitHub。

## 已确认范围

1. 保留现有火山TTS 2.0和安全密钥链路，不更换供应商。
2. 增加可视化音色弹窗、搜索、语言筛选、场景筛选、音色卡片和官方样音试听。
3. 第一版提供约25款真实音色，覆盖中文、英语、日语、西班牙语和印尼语。
4. 选择音色后继续复用现有普通MP3和逐句配音视频流程。
5. 保留手动填写音色ID入口，兼容用户账号中的其他已开通音色。
6. 不复制竞品头像、品牌、图标或文案；使用项目现有暗色绿色视觉系统。

## 技术决定

1. 音色目录、筛选和界面全部放在`src/features/tts`，`App.vue`不新增音色库业务。
2. 官方样音直接播放火山公开示例地址，不消耗用户TTS额度。
3. 音色实际可用范围受用户火山账号权限影响，生成失败时保留现有中文错误和自定义ID兜底。
4. 本次不修改数据库、Rust TTS协议或VideoEngine。

## 预计修改

- 新增：`src/features/tts/voiceCatalog.ts`
- 新增：`src/features/tts/components/VoiceSelectorCard.vue`
- 新增：`src/features/tts/components/VoiceLibraryDialog.vue`
- 新增：`src/features/tts/components/VoiceLibraryFilters.vue`
- 新增：`src/features/tts/components/VoiceCatalogCard.vue`
- 新增：`src/features/tts/voice-library.css`
- 修改：`src/features/tts/components/TtsSettingsPanel.vue`
- 阶段完成后同步任务和架构文档。

## 当前完成情况

1. 已新增25款真实音色目录、搜索、语言/多语种筛选、场景筛选和可视化卡片。
2. 已接入官方样音试听和现有`speaker`字段，保留手动音色ID兜底。
3. 已补充Esc关闭、键盘焦点约束、关闭后焦点返回、空结果和试听错误反馈。
4. 25个官方样音链接检查通过，前端构建和Rust全套检查通过，104项测试全部通过。

## 下一动作

交给用户人工验收。验收通过后提交本地Git，不推GitHub；开始下一项功能前先找用户确认。
