<script setup lang="ts">
import type {
  EncoderCapabilities,
  OutputFrameRate,
  OutputQuality,
  OutputResolution,
  VideoEncoder,
} from "../../types/outputSettings";
import { readSelectedValue } from "./inputHelpers";

const props = defineProps<{
  outputDirectory: string | null;
  outputDirectoryError: string | null;
  isExporting: boolean;
  isProcessing: boolean;
  outputResolution: OutputResolution;
  outputFrameRate: OutputFrameRate;
  outputQuality: OutputQuality;
  outputEncoder: VideoEncoder;
  encoderCapabilities: EncoderCapabilities | null;
  encoderDetectionError: string | null;
  isDetectingEncoders: boolean;
}>();

const emit = defineEmits<{
  selectOutputDirectory: [];
  openOutputDirectory: [];
  exportSelectedVideo: [];
  detectEncoders: [];
  "update:outputResolution": [value: OutputResolution];
  "update:outputFrameRate": [value: OutputFrameRate];
  "update:outputQuality": [value: OutputQuality];
  "update:outputEncoder": [value: VideoEncoder];
}>();

function encoderAvailable(encoder: Exclude<VideoEncoder, "auto">) {
  if (encoder === "cpu") return true;
  return Boolean(
    props.encoderCapabilities?.encoders.some(
      (item) => item.encoder === encoder && item.available,
    ),
  );
}
</script>

<template>
  <div class="output-settings-stack">
    <section class="output-setting-group" aria-labelledby="output-basic-title">
      <div class="output-setting-group__header">
        <div>
          <h3 id="output-basic-title">成片规格</h3>
          <p>推荐保持MP4、1080p附近和标准质量，兼容大多数短视频平台。</p>
        </div>
      </div>

      <div class="output-setting-grid">
        <label class="field">
          <span>输出格式</span>
          <select value="mp4" disabled aria-describedby="output-format-help">
            <option value="mp4">MP4（H.264 + AAC）</option>
          </select>
          <small id="output-format-help">第一版固定MP4，避免不同平台兼容问题。</small>
        </label>

        <label class="field">
          <span>分辨率</span>
          <select
            :value="outputResolution"
            :disabled="isProcessing"
            @change="emit('update:outputResolution', readSelectedValue($event) as OutputResolution)"
          >
            <option value="followCanvas">跟随画布（推荐）</option>
            <option value="hd720">720p（更省空间）</option>
            <option value="fullHd1080">1080p（更清晰）</option>
          </select>
        </label>

        <label class="field">
          <span>帧率</span>
          <select
            :value="outputFrameRate"
            :disabled="isProcessing"
            @change="emit('update:outputFrameRate', readSelectedValue($event) as OutputFrameRate)"
          >
            <option value="source">跟随源视频（推荐）</option>
            <option value="fps24">24 fps</option>
            <option value="fps25">25 fps</option>
            <option value="fps30">30 fps</option>
            <option value="fps50">50 fps</option>
            <option value="fps60">60 fps</option>
          </select>
        </label>

        <label class="field">
          <span>清晰度 / 文件大小</span>
          <select
            :value="outputQuality"
            :disabled="isProcessing"
            @change="emit('update:outputQuality', readSelectedValue($event) as OutputQuality)"
          >
            <option value="compact">省空间</option>
            <option value="standard">标准（推荐）</option>
            <option value="high">高清</option>
          </select>
        </label>
      </div>
    </section>

    <section class="output-setting-group" aria-labelledby="output-encoder-title">
      <div class="output-setting-group__header output-setting-group__header--action">
        <div>
          <h3 id="output-encoder-title">编码方式</h3>
          <p>GPU通常更快；自动模式会优先选择实际测试通过的显卡。</p>
        </div>
        <button class="panel-toggle" type="button" :disabled="isDetectingEncoders || isProcessing" @click="emit('detectEncoders')">
          {{ isDetectingEncoders ? "检测中..." : "重新检测" }}
        </button>
      </div>

      <label class="field">
        <span>视频编码器</span>
        <select
          :value="outputEncoder"
          :disabled="isDetectingEncoders || isProcessing"
          @change="emit('update:outputEncoder', readSelectedValue($event) as VideoEncoder)"
        >
          <option value="auto">自动选择（推荐）</option>
          <option value="cpu">CPU（兼容性最好）</option>
          <option value="nvidia" :disabled="!encoderAvailable('nvidia')">NVIDIA GPU</option>
          <option value="intel" :disabled="!encoderAvailable('intel')">Intel GPU</option>
          <option value="amd" :disabled="!encoderAvailable('amd')">AMD GPU</option>
        </select>
      </label>

      <p v-if="isDetectingEncoders" class="output-encoder-message" role="status">正在进行极小编码测试，通常只需几秒。</p>
      <p v-else-if="encoderDetectionError" class="error-text" role="alert">{{ encoderDetectionError }}</p>
      <p v-else-if="encoderCapabilities" class="output-encoder-message" role="status">
        {{ encoderCapabilities.message }}
      </p>

      <ul v-if="encoderCapabilities" class="encoder-capability-list" aria-label="编码器检测结果">
        <li v-for="item in encoderCapabilities.encoders" :key="item.encoder">
          <span>{{ item.label }}</span>
          <strong :class="item.available ? 'is-available' : 'is-unavailable'">
            {{ item.available ? "可用" : "不可用" }}
          </strong>
        </li>
      </ul>
    </section>

    <section class="output-setting-group" aria-labelledby="output-directory-title">
      <div class="output-setting-group__header">
        <div>
          <h3 id="output-directory-title">保存位置</h3>
          <p>所有最终成片都会保存到这个文件夹。</p>
        </div>
      </div>
      <p v-if="outputDirectory" class="output-path">{{ outputDirectory }}</p>
      <p v-else class="empty-text">请选择导出结果保存位置。</p>
      <div class="output-directory-actions">
        <button class="ghost-button" type="button" @click="emit('selectOutputDirectory')">选择目录</button>
        <button class="ghost-button" type="button" :disabled="!outputDirectory" @click="emit('openOutputDirectory')">打开目录</button>
      </div>
      <button class="primary-button primary-button--full" type="button" :disabled="isExporting" @click="emit('exportSelectedVideo')">
        {{ isExporting ? "正在导出..." : "按当前设置导出视频" }}
      </button>
      <p v-if="outputDirectoryError" class="error-text">{{ outputDirectoryError }}</p>
    </section>
  </div>
</template>
