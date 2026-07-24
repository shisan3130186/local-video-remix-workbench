<script setup lang="ts">
import { ref } from "vue";
import type { AccountAction } from "../types";

defineProps<{
  serviceConfigured: boolean;
  activeAction: AccountAction | null;
}>();

const emit = defineEmits<{
  login: [email: string, password: string];
  register: [email: string, password: string, displayName: string];
}>();

const mode = ref<"login" | "register">("login");
const email = ref("");
const password = ref("");
const confirmPassword = ref("");
const displayName = ref("");
const localError = ref<string | null>(null);

function submit() {
  localError.value = null;
  if (!email.value.trim() || !password.value) {
    localError.value = "请填写邮箱和密码。";
    return;
  }
  if (mode.value === "register") {
    if (password.value !== confirmPassword.value) {
      localError.value = "两次输入的密码不一致。";
      return;
    }
    emit("register", email.value.trim(), password.value, displayName.value.trim());
  } else {
    emit("login", email.value.trim(), password.value);
  }
}

function switchMode(nextMode: "login" | "register") {
  mode.value = nextMode;
  localError.value = null;
  password.value = "";
  confirmPassword.value = "";
}
</script>

<template>
  <section class="account-auth" aria-labelledby="account-auth-title">
    <div class="account-auth__brand" aria-hidden="true">S</div>
    <header>
      <h3 id="account-auth-title">{{ mode === "login" ? "登录智剪账号" : "创建智剪账号" }}</h3>
      <p>登录后可以兑换会员，并在其他电脑恢复有效期。</p>
    </header>

    <div class="account-auth__tabs" role="tablist" aria-label="账号操作">
      <button type="button" role="tab" :aria-selected="mode === 'login'" @click="switchMode('login')">登录</button>
      <button type="button" role="tab" :aria-selected="mode === 'register'" @click="switchMode('register')">注册</button>
    </div>

    <form @submit.prevent="submit">
      <label v-if="mode === 'register'" for="account-display-name">
        <span>昵称</span>
        <input id="account-display-name" v-model="displayName" type="text" maxlength="24" autocomplete="nickname" placeholder="不填则自动生成" />
      </label>
      <label for="account-email">
        <span>邮箱</span>
        <input id="account-email" v-model="email" type="email" maxlength="254" autocomplete="email" placeholder="请输入常用邮箱" required />
      </label>
      <label for="account-password">
        <span>密码</span>
        <input id="account-password" v-model="password" type="password" minlength="8" maxlength="128" :autocomplete="mode === 'login' ? 'current-password' : 'new-password'" placeholder="至少8个字符" required />
      </label>
      <label v-if="mode === 'register'" for="account-confirm-password">
        <span>确认密码</span>
        <input id="account-confirm-password" v-model="confirmPassword" type="password" minlength="8" maxlength="128" autocomplete="new-password" placeholder="再次输入密码" required />
      </label>
      <p v-if="localError" class="membership-feedback membership-feedback--error" role="alert">{{ localError }}</p>
      <button class="account-auth__submit" type="submit" :disabled="!serviceConfigured || activeAction !== null">
        {{ activeAction === "login" || activeAction === "register" ? "正在验证..." : mode === "login" ? "登录" : "注册并登录" }}
      </button>
    </form>

    <p v-if="!serviceConfigured" class="account-auth__service-note">账号服务尚未部署，完成Cloudflare配置后即可注册登录。</p>
  </section>
</template>
