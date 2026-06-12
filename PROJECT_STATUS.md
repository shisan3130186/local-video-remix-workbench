# PROJECT_STATUS.md

## 当前版本

v0.2.0-dev

## 项目定位

本地桌面端 AI 视频批量混剪工作台。

## 当前阶段

阶段二：基础混剪能力开发中。

当前核心目标：

```text
视频切片 → 随机片段抽取 → 片段拼接 → 基础混剪导出
```

## 已完成

- [x] 首页 UI
- [x] 视频导入按钮
- [x] 基础页面布局
- [x] 项目整体开发框架规划
- [x] FFmpeg环境检测
- [x] 单视频/多视频文件导入
- [x] 视频信息读取
- [x] 文件夹导入
- [x] 视频预览
- [x] 输出目录选择
- [x] 基础导出任务
- [x] 基础导出日志
- [x] V0.1 主链路人工验收通过
- [x] 固定时长视频切片
- [x] 随机片段抽取
- [x] 片段拼接导出
- [x] 批量生成混剪结果
- [x] 基础水平镜像
- [x] 桌面软件工作台 UI 框架升级
- [x] 基础变速

## 当前阻塞

暂无。

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

日期：2026-06-12
完成：实现基础变速。用户可以在混剪参数中设置 0.5x 到 2.0x 的变速倍数，单次片段拼接和批量生成都会通过 VideoEngine 应用对应变速。
涉及文件：
- src-tauri/src/video_engine/mix.rs
- src-tauri/src/lib.rs
- src/services/videoMixService.ts
- src/App.vue
- PROJECT_STATUS.md
- TODO_NEXT.md
验证情况：`corepack pnpm build` 已通过；设置 `CARGO_HTTP_CHECK_REVOKE=false` 后 `cargo check` 已通过；已检查变速参数由页面传入，FFmpeg 滤镜仍只在 VideoEngine 中拼接。
人工验收：用户已确认基础变速测试完美通过。
遗留问题：暂无。

## 下一步建议

1. 继续开发 V0.2 裁剪比例
2. 裁剪比例先支持原画、9:16 竖屏、1:1 方屏、16:9 横屏
3. 保持裁剪逻辑通过 VideoEngine 封装，不让页面组件直接拼接 FFmpeg 命令
---

## 2026-06-12 补充记录：视频比例画布适配

状态：已开发，待用户人工验收。

本次实际实现的产品逻辑：

1. 不是自由裁剪。
2. 不是拖拽缩放裁剪框。
3. 选择 9:16、1:1、16:9 时，保留完整原视频画面。
4. 目标比例外的空白区域用黑边或模糊背景补齐。
5. 预览区使用目标比例画布展示，前景播放器仍保留原生 controls。
6. 基础导出、片段拼接导出、批量生成混剪结果都接入同一套画布参数。

涉及文件：

- src/App.vue
- src/styles.css
- src/services/videoMixService.ts
- src/services/videoRenderService.ts
- src-tauri/src/lib.rs
- src-tauri/src/video_engine/canvas.rs
- src-tauri/src/video_engine/render.rs
- src-tauri/src/video_engine/mix.rs

验证情况：

- `corepack pnpm build` 已通过。
- `cargo check` 已通过。
- `corepack pnpm tauri dev` 已启动；首次启动时发现旧开发进程占用 1420 端口，已结束旧进程后重新启动。

待人工验收：

1. 原画模式预览和导出不改变比例。
2. 9:16 黑边模式保留完整画面，导出为竖屏画布。
3. 9:16 模糊背景模式保留完整画面，导出为竖屏画布。
4. 1:1 和 16:9 模式导出比例正确。
5. 播放器底部播放、暂停、时长条仍可用。
6. 画布参数能与水平镜像、基础变速一起使用。
