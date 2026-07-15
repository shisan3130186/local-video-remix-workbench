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

```text
materials/
├─ components   素材库和片段列表
├─ services     文件夹扫描、元数据读取和多视频切片工作流
├─ useMaterialCovers.ts 素材封面状态
├─ useMaterials.ts 素材与切片主流程
└─ index.ts     对外统一出口
```

```text
remix-export/
├─ components   批量结果界面
├─ services     基础导出和随机/分类抽取规则
├─ useBasicExport.ts 基础导出状态
├─ useRemixGeneration.ts 普通混剪与批量生成
├─ remixResultMessages.ts 统一结果日志
└─ index.ts     对外统一出口
```

```text
tts/
├─ components   TTS配置、逐句配音、原声保留、生成状态和音频试听
├─ services     普通TTS、逐句临时TTS和配音视频Tauri调用
├─ types.ts     配置状态、生成结果、逐句输入、原声设置和字词时间
├─ useTts.ts    普通MP3、逐句配音视频和原声参数主流程
└─ index.ts     对外统一出口
```

边界规则：

1. `App.vue` 只负责组合功能、页面级导航和少量跨功能状态。
2. 业务流程放 composable，不把网络、文件或 AI 流程直接堆在组件模板旁。
3. 只有多个功能共同使用的内容才放 `src/types`、`src/constants`、`src/composables`。
4. FFmpeg 命令仍然只能放在 Rust VideoEngine。
5. 单个 Vue 组件超过约 200 行时，应先判断能否拆成容器组件和展示组件。
