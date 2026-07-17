import type { SegmentCategoryOption } from "../services/videoMixService";
import type { ModuleCard } from "../types/workbench";

export const WORKBENCH_MODULE_CARDS: ModuleCard[] = [
  {
    title: "AI 智能混剪",
    category: "创作中心",
    description: "导入素材后，按切片、抽取、拼接和批量生成形成本地混剪流程。",
    tags: ["视频混剪", "批量生成", "画布适配"],
    available: true,
  },
  {
    title: "视频效果处理",
    category: "效率工具",
    description: "面向镜像、变速、画布比例和背景填充的本地视频处理入口。",
    tags: ["镜像", "变速", "模糊背景"],
    available: true,
  },
  {
    title: "视频混剪",
    category: "创作中心",
    description: "固定切片、随机抽取、拼接抽中片段和批量导出。",
    tags: ["切片", "随机抽取", "拼接"],
    available: true,
  },
  {
    title: "分类混剪",
    category: "创作中心",
    description: "按素材分类和规则生成不同混剪版本。",
    tags: ["分类素材", "规则混剪"],
    available: true,
  },
  {
    title: "文案改写",
    category: "效率工具",
    description: "后续用于文案多版本整理和创意表达。",
    tags: ["文案", "多版本"],
    available: false,
  },
  {
    title: "视频内容提炼",
    category: "自动化",
    description: "自动提取片段主题、可见卖点、动作、标签和镜头类型。",
    tags: ["内容提炼", "素材标签"],
    available: true,
  },
];

export const SEGMENT_CATEGORY_OPTIONS: SegmentCategoryOption[] = [
  { key: "hook", label: "开头钩子" },
  { key: "product", label: "产品展示" },
  { key: "usage", label: "使用过程" },
  { key: "detail", label: "细节特写" },
  { key: "result", label: "效果展示" },
  { key: "ending", label: "结尾引导" },
  { key: "talking", label: "人物口播" },
  { key: "environment", label: "环境镜头" },
];
