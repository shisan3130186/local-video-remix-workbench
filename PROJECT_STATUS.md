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
完成：实现固定时长视频切片。用户选择已导入视频、输出目录和切片秒数后，可以生成多个 mp4 片段，并在页面看到切片开始、切片中、切片成功或失败日志。
涉及文件：
- src-tauri/src/lib.rs
- src-tauri/src/video_engine/mod.rs
- src-tauri/src/video_engine/split.rs
- src/services/videoSplitService.ts
- src/App.vue
- src/styles.css
- DECISIONS.md
- PROJECT_STATUS.md
- TODO_NEXT.md
验证情况：`corepack pnpm build` 已通过；设置 `CARGO_HTTP_CHECK_REVOKE=false` 后 `cargo check` 已通过；已用测试素材执行 FFmpeg 切片参数验证，输出到 `E:\Workspace\测试结果库\Project_03_本地短视频批量混剪工作台\manual_split_check`；已检查页面组件没有直接拼接 FFmpeg / FFprobe 命令。
遗留问题：暂无。

## 下一步建议

1. 测试固定时长视频切片功能
2. 开发随机片段抽取
