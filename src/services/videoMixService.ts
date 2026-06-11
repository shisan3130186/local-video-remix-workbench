export function pickRandomSegments(
  segmentPaths: string[],
  pickCount: number,
): string[] {
  if (!Number.isInteger(pickCount) || pickCount <= 0) {
    throw new Error("随机抽取数量必须大于 0。");
  }

  if (segmentPaths.length === 0) {
    throw new Error("请先完成一次视频切片。");
  }

  if (pickCount > segmentPaths.length) {
    throw new Error(`抽取数量不能超过当前片段数量：${segmentPaths.length}。`);
  }

  return shuffleSegments(segmentPaths).slice(0, pickCount);
}

function shuffleSegments(segmentPaths: string[]) {
  const shuffled = [...segmentPaths];

  for (let index = shuffled.length - 1; index > 0; index -= 1) {
    const randomIndex = Math.floor(Math.random() * (index + 1));
    [shuffled[index], shuffled[randomIndex]] = [shuffled[randomIndex], shuffled[index]];
  }

  return shuffled;
}
