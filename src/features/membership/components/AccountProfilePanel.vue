<script setup lang="ts">
import { computed, ref } from "vue";
import type { AccountAction, AccountStatus } from "../types";

const props = defineProps<{
  status: AccountStatus;
  activeAction: AccountAction | null;
}>();

const emit = defineEmits<{
  redeem: [code: string, allowDeviceRebind: boolean];
  changePassword: [currentPassword: string, newPassword: string];
  refresh: [];
  logout: [];
}>();

const redemptionCode = ref("");
const currentPassword = ref("");
const newPassword = ref("");
const confirmPassword = ref("");
const passwordError = ref<string | null>(null);
const pendingLogout = ref(false);
const pendingRedemption = ref<string | null>(null);

const memberLabel = computed(() => props.status.memberActive ? "智剪会员" : "普通用户");
const expiryLabel = computed(() => {
  if (!props.status.expiresAt) return "未开通";
  return formatTime(props.status.expiresAt);
});
const expiryState = computed(() => {
  if (props.status.membershipState === "active") return { label: "有效", modifier: "active" };
  if (props.status.membershipState === "expired") return { label: "已过期", modifier: "expired" };
  return null;
});

function submitRedeem() {
  const code = redemptionCode.value.trim();
  if (!code) return;
  pendingRedemption.value = code;
}

function confirmRedeem() {
  if (!pendingRedemption.value) return;
  emit("redeem", pendingRedemption.value, true);
  pendingRedemption.value = null;
}

function submitPassword() {
  passwordError.value = null;
  if (newPassword.value !== confirmPassword.value) {
    passwordError.value = "两次输入的新密码不一致。";
    return;
  }
  emit("changePassword", currentPassword.value, newPassword.value);
}

function requestLogout() {
  if (!pendingLogout.value) {
    pendingLogout.value = true;
    return;
  }
  emit("logout");
}

function formatTime(value: number) {
  const parts = new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).formatToParts(new Date(value * 1000));
  const values = Object.fromEntries(parts.map((part) => [part.type, part.value]));
  return `${values.year}/${values.month}/${values.day} ${values.hour}:${values.minute}:${values.second}`;
}
</script>

<template>
  <section class="account-profile" aria-labelledby="account-profile-title">
    <div class="account-profile__card">
      <div class="account-profile__identity">
        <span aria-hidden="true">{{ status.displayName?.slice(0, 1) || "智" }}</span>
        <div>
          <h3 id="account-profile-title">{{ status.displayName }}</h3>
          <small :class="{ 'account-profile__member': status.memberActive }">{{ memberLabel }}</small>
        </div>
        <button type="button" :disabled="activeAction !== null" @click="emit('refresh')">刷新状态</button>
      </div>
      <dl>
        <div><dt>邮箱</dt><dd>{{ status.email }}</dd></div>
        <div>
          <dt>有效期至</dt>
          <dd class="account-profile__expiry-row">
            <span>{{ expiryLabel }}</span>
            <span
              v-if="expiryState"
              class="account-profile__expiry-badge"
              :class="`account-profile__expiry-badge--${expiryState.modifier}`"
            >{{ expiryState.label }}</span>
          </dd>
        </div>
        <div>
          <dt>设备绑定</dt>
          <dd :class="{ 'account-profile__device--warning': status.deviceBound && !status.deviceMatch }">
            {{ !status.deviceBound ? "兑换后绑定当前设备" : status.deviceMatch ? "当前设备已绑定" : "当前设备未绑定，兑换后可换绑" }}
          </dd>
        </div>
      </dl>
    </div>

    <details class="account-profile__section" open>
      <summary>卡密兑换</summary>
      <form @submit.prevent="submitRedeem">
        <div class="account-profile__inline-form">
          <input v-model="redemptionCode" type="text" autocomplete="off" spellcheck="false" placeholder="请输入SCUT开头的兑换码" :disabled="activeAction !== null" />
          <button type="submit" :disabled="activeAction !== null || !redemptionCode.trim()">
            {{ activeAction === "redeem" ? "兑换中..." : "兑换" }}
          </button>
        </div>
        <small>兑换成功后，会员天数会直接增加到当前账号。</small>
      </form>
    </details>

    <div v-if="pendingRedemption" class="account-profile__confirm" role="dialog" aria-modal="true" aria-labelledby="device-binding-title">
      <div class="account-profile__confirm-card">
        <h3 id="device-binding-title">确认兑换并绑定设备</h3>
        <p>兑换后会员账号将绑定到此设备，如需在其他设备使用需进行换绑（每月限2次），确定要继续吗？</p>
        <div class="account-profile__confirm-actions">
          <button type="button" @click="pendingRedemption = null">取消</button>
          <button type="button" class="account-profile__confirm-primary" :disabled="activeAction !== null" @click="confirmRedeem">确定继续</button>
        </div>
      </div>
    </div>

    <details class="account-profile__section">
      <summary>修改密码</summary>
      <form class="account-profile__password" @submit.prevent="submitPassword">
        <label><span>当前密码</span><input v-model="currentPassword" type="password" autocomplete="current-password" minlength="8" maxlength="128" required /></label>
        <label><span>新密码</span><input v-model="newPassword" type="password" autocomplete="new-password" minlength="8" maxlength="128" required /></label>
        <label><span>确认新密码</span><input v-model="confirmPassword" type="password" autocomplete="new-password" minlength="8" maxlength="128" required /></label>
        <p v-if="passwordError" class="membership-feedback membership-feedback--error" role="alert">{{ passwordError }}</p>
        <button type="submit" :disabled="activeAction !== null">{{ activeAction === "changePassword" ? "修改中..." : "确认修改" }}</button>
      </form>
    </details>

    <footer>
      <button type="button" class="account-profile__logout" :class="{ 'account-profile__logout--confirm': pendingLogout }" :disabled="activeAction !== null" @click="requestLogout">
        {{ pendingLogout ? "再次点击确认退出" : "退出登录" }}
      </button>
    </footer>
  </section>
</template>
