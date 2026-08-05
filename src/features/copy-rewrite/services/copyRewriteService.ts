import { invoke } from "@tauri-apps/api/core";
import type { RewriteRequest, RewriteResult } from "../types";

export function rewriteScripts(input: RewriteRequest): Promise<RewriteResult> {
  return invoke<RewriteResult>("rewrite_scripts", { input });
}
