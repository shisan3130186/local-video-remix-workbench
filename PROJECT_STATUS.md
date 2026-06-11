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
- [ ] 视频信息读取
- [ ] 视频预览
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
完成：新增 V0.1 第一个功能，软件启动时检测 ffmpeg / ffprobe 是否可调用，并显示版本号或明确错误提示。
涉及文件：
- PROJECT_STATUS.md
- TODO_NEXT.md
- package.json
- index.html
- tsconfig.json
- vite.config.ts
- src/main.ts
- src/App.vue
- src/styles.css
- src/services/videoProbeService.ts
- src/types/videoProbe.ts
- src-tauri/Cargo.toml
- src-tauri/build.rs
- src-tauri/tauri.conf.json
- src-tauri/src/main.rs
- src-tauri/src/lib.rs
- src-tauri/src/video_engine/mod.rs
- src-tauri/src/video_engine/probe.rs
遗留问题：本机尚未安装前端依赖，完整运行测试需要先执行依赖安装。

## 下一步建议

1. 完成视频信息读取
2. 完成单视频导入
3. 完成导入视频后的预览
