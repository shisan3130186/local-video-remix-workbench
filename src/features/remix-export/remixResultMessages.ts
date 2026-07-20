import type { MixVideoResult, RemixExportSettings } from "../../services/videoMixService";

export function formatRemixCanvasLog(result: MixVideoResult) {
  return `混剪输出：比例 ${result.outputAspectRatio}，分辨率 ${result.outputResolution}，${result.outputFrameRate}，${result.outputQuality}，${result.outputEncoder}，约 ${result.outputVideoBitrateKbps} kbps，背景方式 ${result.backgroundMode}，已应用到混剪导出：${
    result.appliedToRemixExport ? "是" : "否"
  }。`;
}

export function formatSmoothRemixLog(result: MixVideoResult) {
  if (!result.smoothRemixEnabled) {
    return "平滑混剪：未开启。";
  }

  return `平滑混剪：已开启，过滤过短片段 ${result.skippedShortSegmentCount} 个。`;
}

export function buildMixOptionSummary(settings: RemixExportSettings) {
  const options: string[] = [];
  const effects = settings.videoEffectSettings;
  const pip = settings.pictureInPictureSettings;
  const bgm = settings.bgmSettings;
  const watermark = settings.watermarkSettings;
  const removal = settings.watermarkRemovalSettings;

  if (settings.applyHorizontalMirror) options.push("已应用水平镜像");
  if (effects.verticalMirror) options.push("已应用垂直镜像");
  if (settings.smoothRemixEnabled) options.push("已应用平滑混剪");
  if (effects.rotation !== "none") options.push("已应用旋转");

  if (
    Math.abs(effects.brightness) > 0.001 ||
    Math.abs(effects.contrast - 1) > 0.001 ||
    Math.abs(effects.saturation - 1) > 0.001
  ) {
    options.push("已应用画面调整");
  }

  if (Math.abs(effects.scale - 1) > 0.001) {
    options.push(`已应用 ${effects.scale.toFixed(2)}x 轻微缩放`);
  }

  if (pip.enabled) options.push("已应用画中画");
  if (watermark.enabled) {
    options.push(watermark.kind === "text" ? "已添加文字水印" : "已添加图片水印");
  }
  if (removal.enabled) options.push("已处理原水印区域");

  if (bgm.enabled) {
    const fadeSummary =
      bgm.fadeInSeconds > 0 || bgm.fadeOutSeconds > 0
        ? `，淡入 ${bgm.fadeInSeconds.toFixed(1)} 秒，淡出 ${bgm.fadeOutSeconds.toFixed(1)} 秒`
        : "";
    options.push(`已添加 BGM${fadeSummary}`);
  }

  if (Math.abs(settings.playbackSpeed - 1) > 0.001) {
    options.push(`已应用 ${settings.playbackSpeed.toFixed(2)}x 变速`);
  }

  return options.length > 0 ? `，${options.join("，")}。` : "。";
}
