export interface RewriteRequest {
  scripts: string[];
  style: "conservative" | "creative" | "custom";
  targetLength: "shorter" | "similar" | "longer";
  customPrompt: string;
  count: number;
}

export interface RewriteResultItem {
  sourceIndex: number;
  sourceText: string;
  versions: string[];
}

export interface RewriteResult {
  items: RewriteResultItem[];
}
