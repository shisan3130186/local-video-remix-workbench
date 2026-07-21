<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { ApiConfigStatus } from "../../api-config";
import type { FfmpegEnvironmentResult } from "../../../types/videoProbe";
import type { DiagnosticInfo } from "../types";

const props = defineProps<{
  visible: boolean;
  startStep: number;
  environment: FfmpegEnvironmentResult | null;
  apiStatus: ApiConfigStatus | null;
  diagnostics: DiagnosticInfo | null;
  isRefreshing: boolean;
  error: string | null;
  reportFeedback: string | null;
}>();

const emit = defineEmits<{
  close: [];
  refresh: [];
  openApiConfig: [];
  createReport: [];
  openLogs: [];
  openLegal: [];
}>();

const step = ref(0);
const closeButton = ref<HTMLButtonElement | null>(null);
const steps = ["运行环境", "AI与语音", "隐私与许可", "完成准备"];

watch(
  () => [props.visible, props.startStep] as const,
  ([visible, startStep]) => {
    if (!visible) return;
    step.value = Math.min(Math.max(startStep, 0), steps.length - 1);
    void nextTick(() => closeButton.value?.focus());
  },
  { immediate: true },
);

function previousStep() {
  step.value = Math.max(0, step.value - 1);
}

function nextStep() {
  step.value = Math.min(steps.length - 1, step.value + 1);
}

function shortVersion(value: string | null | undefined) {
  if (!value) return "未读取版本";
  return value.replace(/ Copyright.*$/i, "");
}
</script>

<template>
  <div
    v-if="visible"
    class="first-launch-backdrop"
    role="presentation"
    @keydown.esc="emit('close')"
  >
    <section class="first-launch-wizard" role="dialog" aria-modal="true" aria-labelledby="first-launch-title">
      <aside class="first-launch-wizard__rail">
        <div class="first-launch-wizard__brand">
          <span>智</span>
          <div><strong>智剪 SmartCut</strong><small>首次使用设置</small></div>
        </div>
        <ol>
          <li
            v-for="(label, index) in steps"
            :key="label"
            :class="{
              'first-launch-step--active': step === index,
              'first-launch-step--done': step > index,
            }"
          >
            <b>{{ step > index ? "✓" : index + 1 }}</b>
            <span>{{ label }}</span>
          </li>
        </ol>
        <p>这套向导可以从顶部“新手教程”随时重新打开。</p>
      </aside>

      <div class="first-launch-wizard__body">
        <header>
          <div>
            <small>步骤 {{ step + 1 }} / {{ steps.length }}</small>
            <h2 id="first-launch-title">{{ steps[step] }}</h2>
          </div>
          <button ref="closeButton" class="first-launch-close" type="button" aria-label="关闭首次启动向导" @click="emit('close')">×</button>
        </header>

        <main>
          <section v-if="step === 0" class="first-launch-page">
            <div class="first-launch-intro">
              <h3>先确认本机视频引擎</h3>
              <p>安装包已经内置FFmpeg和FFprobe，普通用户不需要打开终端或单独配置路径。</p>
            </div>
            <div class="readiness-grid">
              <article :class="{ 'readiness-card--ok': environment?.ffmpeg.available }">
                <span>{{ environment?.ffmpeg.available ? "✓" : "…" }}</span>
                <div><strong>FFmpeg</strong><small>{{ shortVersion(environment?.ffmpeg.version) }}</small></div>
                <em>{{ environment?.ffmpeg.bundled ? "安装包内置" : environment?.ffmpeg.available ? "系统可用" : "等待检测" }}</em>
              </article>
              <article :class="{ 'readiness-card--ok': environment?.ffprobe.available }">
                <span>{{ environment?.ffprobe.available ? "✓" : "…" }}</span>
                <div><strong>FFprobe</strong><small>{{ shortVersion(environment?.ffprobe.version) }}</small></div>
                <em>{{ environment?.ffprobe.bundled ? "安装包内置" : environment?.ffprobe.available ? "系统可用" : "等待检测" }}</em>
              </article>
            </div>
            <div class="first-launch-note" :class="{ 'first-launch-note--ok': environment?.available }">
              <strong>{{ environment?.available ? "本地视频能力已就绪" : "正在检查视频引擎" }}</strong>
              <span>{{ environment?.message ?? "首次检查通常只需要几秒。" }}</span>
            </div>
            <button class="ghost-button first-launch-inline-action" type="button" :disabled="isRefreshing" @click="emit('refresh')">
              {{ isRefreshing ? "正在重新检测..." : "重新检测" }}
            </button>
          </section>

          <section v-else-if="step === 1" class="first-launch-page">
            <div class="first-launch-intro">
              <h3>检查AI、配音和语音识别</h3>
              <p>密钥只需填写一次并由Windows加密保存。没有配置也可以先使用本地视频工具。</p>
            </div>
            <div class="cloud-readiness-list">
              <article :class="{ 'cloud-readiness--ok': apiStatus?.aiConfigured }">
                <b>AI</b><span><strong>画面理解与文案分镜</strong><small>需要方舟API Key和模型接入点</small></span><em>{{ apiStatus?.aiConfigured ? "已配置" : "待配置" }}</em>
              </article>
              <article :class="{ 'cloud-readiness--ok': apiStatus?.ttsConfigured }">
                <b>声</b><span><strong>AI配音</strong><small>需要火山语音密钥</small></span><em>{{ apiStatus?.ttsConfigured ? "已配置" : "待配置" }}</em>
              </article>
              <article :class="{ 'cloud-readiness--ok': apiStatus?.ttsConfigured }">
                <b>识</b><span><strong>ASR语音识别</strong><small>与AI配音共用火山语音密钥</small></span><em>{{ apiStatus?.ttsConfigured ? "已配置" : "待配置" }}</em>
              </article>
            </div>
            <div class="first-launch-actions-row">
              <button class="primary-button" type="button" @click="emit('openApiConfig')">填写或检查密钥</button>
              <button class="ghost-button" type="button" :disabled="isRefreshing" @click="emit('refresh')">刷新状态</button>
            </div>
          </section>

          <section v-else-if="step === 2" class="first-launch-page">
            <div class="first-launch-intro">
              <h3>大白话隐私说明</h3>
              <p>智剪默认在本机处理和保存项目。只有主动使用云端AI功能时，必要数据才会发送给你配置的服务商。</p>
            </div>
            <div class="privacy-points">
              <article><b>本地</b><span><strong>素材与项目默认保存在本机</strong><small>不会自动上传完整素材库。</small></span></article>
              <article><b>云端</b><span><strong>AI功能只发送完成任务所需数据</strong><small>例如图片帧、文案或待识别音频。</small></span></article>
              <article><b>密钥</b><span><strong>API密钥由Windows当前用户加密</strong><small>不会进入日志、项目文件或诊断报告。</small></span></article>
              <article><b>日志</b><span><strong>诊断信息默认只保存在本机</strong><small>不记录视频内容、文案正文和完整密钥。</small></span></article>
            </div>
            <div class="license-strip">
              <span><strong>FFmpeg</strong><small>GPL-3.0，许可证和来源说明已随安装包提供。</small></span>
              <span><strong>第三方组件</strong><small>Tauri、Vue及主要依赖均提供组件说明。</small></span>
              <button class="ghost-button" type="button" @click="emit('openLegal')">查看完整说明</button>
            </div>
          </section>

          <section v-else class="first-launch-page">
            <div class="first-launch-intro">
              <h3>准备完成，可以开始创作</h3>
              <p>如果后续遇到无法启动或导出失败，可以一键生成不含密钥和视频内容的诊断报告。</p>
            </div>
            <div class="final-readiness">
              <span :class="{ 'final-readiness--ok': environment?.available }"><b>{{ environment?.available ? "✓" : "!" }}</b> 本地视频引擎</span>
              <span :class="{ 'final-readiness--ok': apiStatus?.aiConfigured }"><b>{{ apiStatus?.aiConfigured ? "✓" : "-" }}</b> AI画面理解</span>
              <span :class="{ 'final-readiness--ok': apiStatus?.ttsConfigured }"><b>{{ apiStatus?.ttsConfigured ? "✓" : "-" }}</b> 配音与语音识别</span>
              <span :class="{ 'final-readiness--ok': diagnostics }"><b>{{ diagnostics ? "✓" : "…" }}</b> 日志与诊断</span>
            </div>
            <div class="diagnostic-card">
              <div>
                <strong>诊断目录</strong>
                <small>{{ diagnostics?.logsDirectory ?? "正在读取..." }}</small>
                <small v-if="diagnostics?.latestCrashLog">发现历史崩溃日志，可在目录中查看。</small>
              </div>
              <div class="diagnostic-card__actions">
                <button class="ghost-button" type="button" @click="emit('createReport')">生成诊断报告</button>
                <button class="ghost-button" type="button" @click="emit('openLogs')">打开诊断目录</button>
              </div>
              <p v-if="reportFeedback" role="status">{{ reportFeedback }}</p>
            </div>
          </section>

          <p v-if="error" class="first-launch-error" role="alert">{{ error }}</p>
        </main>

        <footer>
          <button class="ghost-button" type="button" :disabled="step === 0" @click="previousStep">上一步</button>
          <span>{{ step === 1 && !apiStatus?.aiConfigured ? "云端功能可以稍后配置" : "" }}</span>
          <button v-if="step < steps.length - 1" class="primary-button" type="button" @click="nextStep">下一步</button>
          <button v-else class="primary-button" type="button" @click="emit('close')">完成并进入智剪</button>
        </footer>
      </div>
    </section>
  </div>
</template>
