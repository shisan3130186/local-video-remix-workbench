import { save } from "@tauri-apps/plugin-dialog";
import { computed, ref, watch, type Ref } from "vue";
import { writePosterImage } from "./services/posterExportService";

export interface PosterTemplate { id: string; name: string; background: string; primary: string; secondary: string; accent: string; }

export const posterTemplates: PosterTemplate[] = [
  { id: "signal", name: "信号蓝", background: "#101722", primary: "#f4f7fb", secondary: "#9fb1c8", accent: "#5b8def" },
  { id: "ember", name: "余烬", background: "#201715", primary: "#fff6ed", secondary: "#d9b7a7", accent: "#e76f51" },
  { id: "mint", name: "薄荷", background: "#10201d", primary: "#eefcf7", secondary: "#a5c9bd", accent: "#48b89f" },
  { id: "mono", name: "黑白", background: "#121315", primary: "#f3f3f1", secondary: "#aaaaa5", accent: "#dadad5" },
  { id: "berry", name: "莓果", background: "#21131b", primary: "#fff1f6", secondary: "#d9aaba", accent: "#d35d82" },
  { id: "sun", name: "日光", background: "#222015", primary: "#fffbe8", secondary: "#d9d09c", accent: "#e0bb3f" },
  { id: "ocean", name: "深海", background: "#0b1921", primary: "#edfaff", secondary: "#8db4c4", accent: "#38b9d4" },
  { id: "lime", name: "青柠", background: "#152014", primary: "#f5ffe9", secondary: "#b9cf9d", accent: "#9ad94b" },
  { id: "coral", name: "珊瑚", background: "#241614", primary: "#fff2ec", secondary: "#dcb2a3", accent: "#ff7b61" },
  { id: "violet", name: "夜紫", background: "#191527", primary: "#f5efff", secondary: "#bcb0d5", accent: "#9277dc" },
  { id: "steel", name: "钢蓝", background: "#151a20", primary: "#f0f4f8", secondary: "#aab4be", accent: "#7f9db9" },
  { id: "paper", name: "白纸", background: "#ececeb", primary: "#17191b", secondary: "#626568", accent: "#d54d45" },
  { id: "warning", name: "警示", background: "#171717", primary: "#fffdf4", secondary: "#c8c3ad", accent: "#f0c73c" },
  { id: "coffee", name: "咖啡", background: "#211a17", primary: "#fff7ef", secondary: "#cbb5a5", accent: "#bf805d" },
  { id: "ice", name: "冰川", background: "#132125", primary: "#effeff", secondary: "#9bc5ca", accent: "#67d6d9" },
  { id: "rose", name: "玫红", background: "#24131a", primary: "#fff0f5", secondary: "#d2a0b1", accent: "#ee4f83" },
  { id: "sand", name: "沙金", background: "#211e17", primary: "#fff9e9", secondary: "#cbbf9e", accent: "#d7aa52" },
  { id: "forest", name: "森林", background: "#111c17", primary: "#effbf4", secondary: "#93b5a1", accent: "#4fbd7d" },
  { id: "ink", name: "墨色", background: "#111315", primary: "#e8ecef", secondary: "#92989d", accent: "#ffffff" },
  { id: "cobalt", name: "钴蓝", background: "#10172a", primary: "#f2f5ff", secondary: "#a7b3d4", accent: "#5478e8" },
];

export function usePosterMaker(canvas: Ref<HTMLCanvasElement | null>) {
  const text = ref("主标题\n副标题\n行动号召");
  const aspect = ref<"portrait" | "square" | "landscape">("portrait");
  const templateId = ref(posterTemplates[0].id);
  const fontSize = ref(64);
  const letterSpacing = ref(2);
  const lineHeight = ref(1.22);
  const align = ref<CanvasTextAlign>("center");
  const error = ref<string | null>(null);
  const feedback = ref<string | null>(null);
  const template = computed(() => posterTemplates.find((item) => item.id === templateId.value) ?? posterTemplates[0]);

  function render() {
    const target = canvas.value; if (!target) return;
    const dimensions = aspect.value === "portrait" ? [1080, 1920] : aspect.value === "square" ? [1080, 1080] : [1920, 1080];
    target.width = dimensions[0]; target.height = dimensions[1];
    const context = target.getContext("2d"); if (!context) return;
    const current = template.value;
    context.fillStyle = current.background; context.fillRect(0, 0, target.width, target.height);
    const gradient = context.createRadialGradient(target.width * 0.2, target.height * 0.15, 0, target.width * 0.2, target.height * 0.15, target.width * 0.85);
    gradient.addColorStop(0, `${current.accent}55`); gradient.addColorStop(1, "transparent"); context.fillStyle = gradient; context.fillRect(0, 0, target.width, target.height);
    context.strokeStyle = `${current.accent}88`; context.lineWidth = Math.max(6, target.width * 0.008); context.strokeRect(target.width * 0.07, target.height * 0.06, target.width * 0.86, target.height * 0.88);
    const parsedLines = text.value.split(/\r?\n/).filter((line) => line.trim());
    const lines = parsedLines.length > 0 ? parsedLines : ["请输入文案"];
    const scaledFont = fontSize.value * (target.width / 1080); const gap = scaledFont * lineHeight.value;
    context.textAlign = align.value; context.textBaseline = "middle"; context.font = `700 ${scaledFont}px "Microsoft YaHei UI", sans-serif`;
    const x = align.value === "left" ? target.width * 0.13 : align.value === "right" ? target.width * 0.87 : target.width / 2;
    const startY = target.height / 2 - ((lines.length - 1) * gap) / 2;
    lines.forEach((line, index) => { context.fillStyle = index === lines.length - 1 ? current.accent : index === 0 ? current.primary : current.secondary; drawSpacedText(context, line, x, startY + index * gap, letterSpacing.value * (target.width / 1080), align.value); });
  }

  async function exportPng() {
    const target = canvas.value; if (!target) return;
    const path = await save({ defaultPath: "智剪大字报.png", filters: [{ name: "PNG 图片", extensions: ["png"] }] }); if (!path) return;
    error.value = null;
    try { await writePosterImage(path, target.toDataURL("image/png")); feedback.value = "大字报已导出。"; }
    catch (value) { error.value = value instanceof Error ? value.message : String(value); }
  }

  watch([text, aspect, templateId, fontSize, letterSpacing, lineHeight, align], render, { immediate: true });
  return { align, aspect, error, feedback, fontSize, letterSpacing, lineHeight, templateId, text, render, exportPng };
}

function drawSpacedText(context: CanvasRenderingContext2D, text: string, x: number, y: number, spacing: number, align: CanvasTextAlign) {
  if (spacing <= 0) { context.fillText(text, x, y); return; }
  const chars = [...text]; const widths = chars.map((char) => context.measureText(char).width); const total = widths.reduce((sum, width) => sum + width, 0) + spacing * Math.max(0, chars.length - 1);
  let cursor = align === "center" ? x - total / 2 : align === "right" ? x - total : x;
  chars.forEach((char, index) => { context.fillText(char, cursor, y); cursor += widths[index] + spacing; });
}
