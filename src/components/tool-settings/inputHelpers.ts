export function readNumber(event: Event) {
  return Number((event.target as HTMLInputElement).value);
}

export function readChecked(event: Event) {
  return (event.target as HTMLInputElement).checked;
}

export function readSelectedValue(event: Event) {
  return (event.target as HTMLSelectElement).value;
}

export function clampNumber(value: number, min: number, max: number) {
  if (!Number.isFinite(value)) {
    return min;
  }

  return Math.min(max, Math.max(min, value));
}

export function formatFileName(filePath: string | null) {
  return filePath?.split(/[\\/]/).pop() ?? "未选择文件";
}
