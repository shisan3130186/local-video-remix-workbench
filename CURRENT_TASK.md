# CURRENT_TASK.md

> 最后更新：2026-07-17。只记录当前唯一任务、真实中断点和下一动作；完成后就地改写，不追加成长历史日志。

## 当前唯一任务

第一阶段第1项：完成统一任务控制第一版。

包含四项必须一起验收的能力：

1. 用户可以取消正在执行的视频任务。
2. 底部任务栏显示FFmpeg真实处理百分比和当前阶段。
3. 批量成片中失败的条目可以单独重试，不重复生成成功条目。
4. 软件只清理本项目明确标识的临时目录，不碰素材和最终成片。

## 已确认范围

- 覆盖切片、基础导出、普通混剪、AI无配音成片和AI配音成片。
- FFmpeg运行中取消必须真正停止进程，不能只停止界面动画。
- AI/TTS网络请求无法瞬间强杀时，点击取消后在当前请求返回后停止后续步骤。
- 多阶段混剪的百分比由真实FFmpeg时间与当前阶段范围合并计算。
- 第一版不做暂停恢复、并发队列、重启后续跑和后台常驻任务。

## 当前真实状态

状态：代码开发、自动检查和用户人工验收全部通过；准备提交本地Git，不推GitHub。

已完成：

- Rust任务注册、真实进度解析、取消状态和FFmpeg进程终止。
- 本项目专用临时目录、任务结束自动清理和超过24小时遗留清理。
- 前端任务中心、底部真实百分比、取消按钮、失败项重试和手动清理入口。
- 切片、基础导出、普通混剪、AI无配音和AI配音链路接入。
- 前端构建、Rust格式、编译、严格Clippy和81项测试全部通过。

当前Git基线：`c1dd651`。工作区有未提交改动，不推GitHub。

## 预计修改文件

Rust：

- `src-tauri/src/task_runtime.rs`
- `src-tauri/src/temp_storage.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/video_engine/render.rs`
- `src-tauri/src/video_engine/split.rs`
- `src-tauri/src/video_engine/mix.rs`
- `src-tauri/src/video_engine/narrated_mix.rs`
- 可能调整 `src-tauri/src/tts.rs`

前端：

- 新增 `src/features/task-center`
- `src/components/TaskControlBar.vue`
- `src/App.vue`
- `src/features/materials`
- `src/features/remix-export`
- `src/features/ai-remix`
- `src/features/tts`
- 对应样式文件

## 下一动作

1. 提交本地Git，不推GitHub。
2. 下一候选任务为第一阶段第2项“完整输出设置”。
3. 用户确认下一项范围后，再把本文件改写为新的唯一任务。

## 验收标准

1. 处理视频时显示0～100%的动态百分比，不能使用固定假进度。
2. 点击取消后，当前FFmpeg进程停止，界面显示“已取消”，可重新开始。
3. 取消或失败后不留下半成品结果和本项目临时目录。
4. 批量任务部分失败后显示失败数量和“重试失败项”，成功项不会重复执行。
5. 正常完成时原有成片效果、结果列表和日志不被破坏。
6. 自动检查全部通过，人工验收通过后才创建本地Git提交。
