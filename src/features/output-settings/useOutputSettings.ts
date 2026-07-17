import { computed, ref } from "vue";
import {
  DEFAULT_OUTPUT_SETTINGS,
  type EncoderCapabilities,
  type OutputFrameRate,
  type OutputQuality,
  type OutputResolution,
  type OutputSettings,
  type VideoEncoder,
} from "../../types/outputSettings";
import { getVideoEncoderCapabilities } from "./services/outputSettingsService";

export function useOutputSettings() {
  const outputResolution = ref<OutputResolution>(DEFAULT_OUTPUT_SETTINGS.resolution);
  const outputFrameRate = ref<OutputFrameRate>(DEFAULT_OUTPUT_SETTINGS.frameRate);
  const outputQuality = ref<OutputQuality>(DEFAULT_OUTPUT_SETTINGS.quality);
  const outputEncoder = ref<VideoEncoder>(DEFAULT_OUTPUT_SETTINGS.encoder);
  const encoderCapabilities = ref<EncoderCapabilities | null>(null);
  const encoderDetectionError = ref<string | null>(null);
  const isDetectingEncoders = ref(false);

  const outputSettings = computed(
    (): OutputSettings => ({
      format: "mp4",
      resolution: outputResolution.value,
      frameRate: outputFrameRate.value,
      quality: outputQuality.value,
      encoder: outputEncoder.value,
    }),
  );

  async function detectEncoders(forceRefresh = true) {
    isDetectingEncoders.value = true;
    encoderDetectionError.value = null;
    try {
      encoderCapabilities.value = await getVideoEncoderCapabilities(forceRefresh);
      if (
        outputEncoder.value !== "auto" &&
        !encoderCapabilities.value.encoders.some(
          (item) => item.encoder === outputEncoder.value && item.available,
        )
      ) {
        outputEncoder.value = "auto";
      }
    } catch (error) {
      encoderDetectionError.value = formatError(error, "GPU编码器检测失败，请使用CPU编码。");
      encoderCapabilities.value = null;
      outputEncoder.value = "cpu";
    } finally {
      isDetectingEncoders.value = false;
    }
  }

  function restoreOutputSettings(settings?: Partial<OutputSettings> | null) {
    outputResolution.value = isOutputResolution(settings?.resolution)
      ? settings.resolution
      : DEFAULT_OUTPUT_SETTINGS.resolution;
    outputFrameRate.value = isOutputFrameRate(settings?.frameRate)
      ? settings.frameRate
      : DEFAULT_OUTPUT_SETTINGS.frameRate;
    outputQuality.value = isOutputQuality(settings?.quality)
      ? settings.quality
      : DEFAULT_OUTPUT_SETTINGS.quality;
    outputEncoder.value = isVideoEncoder(settings?.encoder)
      ? settings.encoder
      : DEFAULT_OUTPUT_SETTINGS.encoder;
  }

  return {
    detectEncoders,
    encoderCapabilities,
    encoderDetectionError,
    isDetectingEncoders,
    outputEncoder,
    outputFrameRate,
    outputQuality,
    outputResolution,
    outputSettings,
    restoreOutputSettings,
  };
}

function isOutputResolution(value: unknown): value is OutputResolution {
  return ["followCanvas", "hd720", "fullHd1080"].includes(String(value));
}

function isOutputFrameRate(value: unknown): value is OutputFrameRate {
  return ["source", "fps24", "fps25", "fps30", "fps50", "fps60"].includes(String(value));
}

function isOutputQuality(value: unknown): value is OutputQuality {
  return ["compact", "standard", "high"].includes(String(value));
}

function isVideoEncoder(value: unknown): value is VideoEncoder {
  return ["auto", "cpu", "nvidia", "intel", "amd"].includes(String(value));
}

function formatError(error: unknown, fallback: string) {
  if (error instanceof Error) return error.message;
  return String(error ?? "").trim() || fallback;
}
