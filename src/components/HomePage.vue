<script setup lang="ts">
import { ref } from "vue";
import type { WorkspaceMode } from "../types/workbench";

const smartCutIconUrl = "/smartcut-icon.svg";

defineProps<{
  showWelcome: boolean;
  hasRecentProject: boolean;
  recentProjectTitle: string;
  recentProjectSummary: string;
  environmentAvailable: boolean | null;
}>();

const emit = defineEmits<{
  openWorkspace: [mode: WorkspaceMode];
  continueProject: [];
  openWelcome: [];
  closeWelcome: [];
}>();

const tutorialWebFeedback = ref<string | null>(null);

function showTutorialWebFeedback() {
  tutorialWebFeedback.value = "教程网页入口已经预留，发布测试版前接入正式帮助中心地址。";
}
</script>

<template>
  <section class="smartcut-home">
    <main class="smartcut-home__main">
      <div class="smartcut-home__inner">
        <header class="smartcut-home__welcome">
          <div>
            <small>智剪 SmartCut</small>
            <h2>开始今天的视频创作</h2>
            <p>选择一种方式开始，后面的分析、配音和导出由智剪一步步带你完成。</p>
          </div>
          <div class="smartcut-home__welcome-actions">
            <button class="ghost-button" type="button" @click="emit('continueProject')">
              打开项目
            </button>
            <button class="primary-button" type="button" @click="emit('openWorkspace', 'ai')">
              ＋ 新建项目
            </button>
          </div>
        </header>

        <section class="smartcut-home__section" aria-labelledby="quick-start-title">
          <div class="smartcut-home__section-heading">
            <div>
              <h3 id="quick-start-title">快速开始</h3>
              <p>不需要先理解全部功能，只选你现在要做的事。</p>
            </div>
          </div>
          <div class="smartcut-start-modes">
            <button type="button" @click="emit('openWorkspace', 'ai')">
              <span class="smartcut-start-modes__icon smartcut-start-modes__icon--active">AI</span>
              <span>
                <strong>AI 智能成片</strong>
                <small>导入素材和文案，自动分析、分镜、配音并生成视频。</small>
              </span>
            </button>
            <button type="button" @click="emit('openWorkspace', 'batch')">
              <span class="smartcut-start-modes__icon">批量</span>
              <span>
                <strong>批量混剪</strong>
                <small>按规则组合多个素材，一次生成多个差异版本。</small>
              </span>
            </button>
            <button type="button" @click="emit('openWorkspace', 'tools')">
              <span class="smartcut-start-modes__icon">工具</span>
              <span>
                <strong>视频工具</strong>
                <small>水印、字幕、音频识别、画面处理和格式转换。</small>
              </span>
            </button>
          </div>
        </section>

        <section class="smartcut-home__section" aria-labelledby="recent-project-title">
          <div class="smartcut-home__section-heading">
            <div>
              <h3 id="recent-project-title">最近项目</h3>
              <p>继续上一次未完成的工作。</p>
            </div>
          </div>

          <button
            v-if="hasRecentProject"
            class="smartcut-recent-project"
            type="button"
            @click="emit('continueProject')"
          >
            <span class="smartcut-recent-project__preview" aria-hidden="true"></span>
            <span class="smartcut-recent-project__copy">
              <strong>{{ recentProjectTitle }}</strong>
              <small>{{ recentProjectSummary }}</small>
            </span>
            <span class="smartcut-recent-project__action">继续编辑 ›</span>
          </button>

          <div v-else class="smartcut-home__empty">
            <span aria-hidden="true">＋</span>
            <div>
              <strong>还没有最近项目</strong>
              <small>新建项目后，素材、文案和设置会自动保存。</small>
            </div>
            <button class="ghost-button" type="button" @click="emit('openWorkspace', 'ai')">开始第一个项目</button>
          </div>
        </section>

        <section class="smartcut-home__section" aria-labelledby="ready-title">
          <div class="smartcut-home__section-heading">
            <div>
              <h3 id="ready-title">本机能力状态</h3>
              <p>常用功能不需要每次打开终端。</p>
            </div>
          </div>
          <div class="smartcut-ready-list">
            <span :class="{ 'smartcut-ready-list__item--pending': environmentAvailable !== true }">
              {{ environmentAvailable === null ? "FFmpeg 检测中" : environmentAvailable ? "FFmpeg 可用" : "FFmpeg 待检查" }}
            </span>
            <span>AI 配置可保存</span>
            <span>TTS / ASR 可用</span>
            <span>项目自动保存</span>
            <span>任务可取消和重试</span>
          </div>
        </section>
      </div>
    </main>

    <aside class="smartcut-guide" aria-label="新手引导">
      <header>
        <img :src="smartCutIconUrl" alt="智剪图标" />
        <h2>第一次使用？</h2>
        <p>跟着三步完成第一条视频。引导可以随时跳过，之后也能从帮助中心重新打开。</p>
      </header>
      <div class="smartcut-guide__steps">
        <article><b>1</b><span><strong>导入你的素材</strong><small>选择视频、图片或整个素材文件夹。</small></span></article>
        <article><b>2</b><span><strong>准备内容</strong><small>输入文案，或导入音频自动识别。</small></span></article>
        <article><b>3</b><span><strong>确认并生成</strong><small>检查分镜、配音和字幕，然后批量输出。</small></span></article>
      </div>
      <div class="smartcut-guide__actions">
        <button class="primary-button" type="button" @click="emit('openWelcome')">开始新手引导</button>
        <button class="ghost-button" type="button" @click="showTutorialWebFeedback">打开教程网页</button>
        <small v-if="tutorialWebFeedback" role="status">{{ tutorialWebFeedback }}</small>
        <small v-else>内置引导断网也能使用</small>
      </div>
    </aside>

    <div
      v-if="showWelcome"
      class="modal-backdrop smartcut-tutorial-backdrop"
      tabindex="-1"
      @click.self="emit('closeWelcome')"
      @keydown.esc="emit('closeWelcome')"
    >
      <section class="smartcut-tutorial" role="dialog" aria-modal="true" aria-labelledby="tutorial-title">
        <header>
          <h2 id="tutorial-title">3分钟认识智剪</h2>
          <button class="modal-close" type="button" aria-label="关闭新手教程" @click="emit('closeWelcome')">×</button>
        </header>
        <div class="smartcut-tutorial__steps">
          <article><small>01</small><h3>选一个创作方式</h3><p>普通用户从AI智能成片开始；有固定规则时使用批量混剪；单项处理进入视频工具。</p></article>
          <article><small>02</small><h3>按四步完成</h3><p>准备素材、文案与配音、确认分镜、输出视频。右侧只显示当前步骤需要的设置。</p></article>
          <article><small>03</small><h3>任务可以后台运行</h3><p>生成过程中可以查看真实进度、取消任务、重试失败条目，临时文件会自动清理。</p></article>
        </div>
        <footer>
          <span>首次启动自动显示一次，之后可从首页重新打开。</span>
          <button class="primary-button" type="button" @click="emit('closeWelcome')">我知道了，开始创作</button>
        </footer>
      </section>
    </div>
  </section>
</template>
