<script setup lang="ts">
import { ref, watch } from "vue";
import { ApiConfigSettingsPanel } from "../../api-config";
import type { AccountAction, AccountStatus } from "../types";
import AccountAuthPanel from "./AccountAuthPanel.vue";
import AccountProfilePanel from "./AccountProfilePanel.vue";
import "../membership.css";

const props = defineProps<{
  visible: boolean;
  status: AccountStatus | null;
  activeAction: AccountAction | null;
  error: string | null;
  feedback: string | null;
}>();

const emit = defineEmits<{
  close: [];
  login: [email: string, password: string];
  register: [email: string, password: string, displayName: string];
  refresh: [];
  redeem: [code: string];
  changePassword: [currentPassword: string, newPassword: string];
  logout: [];
  apiConfigChanged: [];
}>();

const activePage = ref<"profile" | "api">("profile");

watch(() => props.visible, (visible) => {
  if (!visible) activePage.value = "profile";
});

function forwardLogin(email: string, password: string) {
  emit("login", email, password);
}

function forwardRegister(email: string, password: string, displayName: string) {
  emit("register", email, password, displayName);
}

function forwardPasswordChange(currentPassword: string, newPassword: string) {
  emit("changePassword", currentPassword, newPassword);
}
</script>

<template>
  <div v-if="visible" class="membership-backdrop" @click.self="emit('close')">
    <section class="membership-dialog" :class="{ 'membership-dialog--signed-in': status?.signedIn }" role="dialog" aria-modal="true" aria-labelledby="membership-title">
      <header class="membership-dialog__header">
        <h2 id="membership-title">{{ status?.signedIn ? "个人中心" : "智剪账号" }}</h2>
        <button type="button" aria-label="关闭账号窗口" @click="emit('close')">关闭</button>
      </header>

      <template v-if="status?.signedIn">
        <aside class="membership-dialog__nav" aria-label="个人中心导航">
          <button type="button" :class="{ 'is-active': activePage === 'profile' }" @click="activePage = 'profile'">个人信息</button>
          <button type="button" :class="{ 'is-active': activePage === 'api' }" @click="activePage = 'api'">API Key</button>
        </aside>
        <main class="membership-dialog__content">
          <AccountProfilePanel
            v-if="activePage === 'profile'"
            :status="status"
            :active-action="activeAction"
            @redeem="emit('redeem', $event)"
            @change-password="forwardPasswordChange"
            @refresh="emit('refresh')"
            @logout="emit('logout')"
          />
          <ApiConfigSettingsPanel v-else @changed="emit('apiConfigChanged')" />
        </main>
      </template>

      <main v-else class="membership-dialog__auth-content">
        <AccountAuthPanel
          :service-configured="status?.serviceConfigured ?? false"
          :active-action="activeAction"
          @login="forwardLogin"
          @register="forwardRegister"
        />
      </main>

      <div class="membership-dialog__messages">
        <p v-if="error" class="membership-feedback membership-feedback--error" role="alert">{{ error }}</p>
        <p v-if="feedback" class="membership-feedback" role="status">{{ feedback }}</p>
      </div>
    </section>
  </div>
</template>
