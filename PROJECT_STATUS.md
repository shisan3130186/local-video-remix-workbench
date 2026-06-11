# PROJECT_STATUS.md

## 当前版本

v0.1.0

## 项目定位

本地桌面端 AI 视频批量混剪工作台。

## 当前阶段

阶段一：基础视频导入、预览、FFmpeg处理引擎打通。

当前核心目标：

```text
FFmpeg 检测 → 导入素材 → 读取信息 → 视频预览 → 基础导出 → 任务日志
```

## 已完成

- [x] 首页 UI
- [x] 视频导入按钮
- [x] 基础页面布局
- [x] 项目整体开发框架规划
- [x] FFmpeg环境检测
- [x] 单视频/多视频文件导入
- [x] 视频信息读取
- [ ] 视频预览
- [ ] 文件夹导入
- [ ] 基础导出任务

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

日期：2026-06-11
完成：实现视频文件导入和视频信息读取。支持选择 mp4 / mov / avi / mkv 文件，导入后显示素材列表，并通过 VideoEngine 调用 ffprobe 读取文件名、路径、时长、分辨率、帧率、是否有音频和文件大小。
涉及文件：
- package.json
- pnpm-lock.yaml
- PROJECT_STATUS.md
- TODO_NEXT.md
- src-tauri/Cargo.lock
- src-tauri/Cargo.toml
- src-tauri/capabilities/default.json
- src-tauri/src/lib.rs
- src-tauri/src/video_engine/probe.rs
- src/App.vue
- src/services/videoProbeService.ts
- src/styles.css
- src/types/videoProbe.ts
验证情况：`corepack pnpm build` 已通过；`cargo check` 已通过；`corepack pnpm tauri build --debug` 已完成应用编译，后续打包阶段因下载 WiX 工具超时失败。
遗留问题：打包器 WiX 下载超时，不影响本次功能代码编译；后续如需生成安装包再处理。

## 下一步建议

1. 完成文件夹导入
2. 完成导入视频后的预览
3. 完成输出目录选择
