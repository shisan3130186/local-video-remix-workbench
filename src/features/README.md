# 前端功能模块目录规范

后续新增 TTS、图片转视频、去水印和基础视频工具时，优先按功能放入本目录，不再继续扩大 `App.vue`。

推荐结构：

```text
src/features/<feature-name>/
├─ components     该功能专用界面
├─ composables    状态和业务流程
├─ services       Tauri / API 调用
├─ types.ts       该功能专用类型
└─ constants.ts   该功能固定选项
```

当前已落地模块：

```text
ai-remix/
├─ components   AI 分镜展示和人工调整
├─ services     Tauri AI 调用、片段准备和画面理解工作流
├─ types.ts     AI 片段和分镜类型
├─ useAiRemix.ts AI 状态与主流程
└─ index.ts     对外统一出口
```

边界规则：

1. `App.vue` 只负责组合功能、页面级导航和少量跨功能状态。
2. 业务流程放 composable，不把网络、文件或 AI 流程直接堆在组件模板旁。
3. 只有多个功能共同使用的内容才放 `src/types`、`src/constants`、`src/composables`。
4. FFmpeg 命令仍然只能放在 Rust VideoEngine。
5. 单个 Vue 组件超过约 200 行时，应先判断能否拆成容器组件和展示组件。
