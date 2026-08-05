export type TaskLogLevel = "info" | "success" | "error";

export type ToolKey =
  | "apiKeys"
  | "remix"
  | "canvas"
  | "audio"
  | "bgm"
  | "tts"
  | "asr"
  | "subtitleStyle"
  | "watermark"
  | "cover"
  | "entrance"
  | "frame"
  | "effects"
  | "transition"
  | "pip"
  | "adjust"
  | "fusion"
  | "rotate"
  | "mirror"
  | "speed"
  | "zoom"
  | "subtitles"
  | "export";

export type DrawerKey = "logs" | "exports" | "batch" | "scripts";

export type WorkspaceMode = "ai" | "batch" | "tools";

export type FeatureKey =
  | "copyRewrite"
  | "fileRenamer"
  | "subtitleEditor"
  | "posterMaker"
  | "imageToVideo";

export interface ModuleCard {
  title: string;
  category: string;
  description: string;
  tags: string[];
  available: boolean;
}

export interface TaskLogEntry {
  id: number;
  time: string;
  message: string;
  level: TaskLogLevel;
}

export interface GroupedTaskLogEntry extends TaskLogEntry {
  group: string;
}

export type ExportResultType =
  | "基础导出"
  | "拼接导出"
  | "分类混剪"
  | "批量生成"
  | "AI 智能混剪"
  | "AI配音混剪";

export interface ExportResultItem {
  id: number;
  type: ExportResultType;
  path: string;
  time: string;
}
