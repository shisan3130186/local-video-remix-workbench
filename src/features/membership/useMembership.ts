import { computed, ref } from "vue";
import {
  changeAccountPassword,
  getAccountStatus,
  loginAccount,
  logoutAccount,
  redeemMembership,
  refreshAccount,
  registerAccount,
} from "./services/membershipService";
import type { AccountAction, AccountStatus } from "./types";

export function useMembership() {
  const status = ref<AccountStatus | null>(null);
  const isDialogVisible = ref(false);
  const activeAction = ref<AccountAction | null>(null);
  const error = ref<string | null>(null);
  const feedback = ref<string | null>(null);

  const isLoading = computed(() => activeAction.value !== null);
  const topBarText = computed(() => {
    if (status.value?.memberActive) return "会员中心";
    if (status.value?.signedIn) return "个人中心";
    return "登录 / 激活";
  });

  async function loadStatus() {
    return run("refresh", async () => {
      status.value = await getAccountStatus();
      return status.value;
    }, true);
  }

  async function refreshStatus(silent = false) {
    return run("refresh", async () => {
      status.value = await refreshAccount();
      if (!silent) feedback.value = status.value.message;
      return status.value;
    }, silent);
  }

  async function register(email: string, password: string, displayName: string) {
    return run("register", async () => {
      status.value = await registerAccount(email, password, displayName);
      feedback.value = "注册成功，已自动登录。";
      return true;
    });
  }

  async function login(email: string, password: string) {
    return run("login", async () => {
      status.value = await loginAccount(email, password);
      feedback.value = "登录成功。";
      return true;
    });
  }

  async function redeem(code: string) {
    return run("redeem", async () => {
      status.value = await redeemMembership(code);
      feedback.value = "兑换成功，会员有效期已更新。";
      return true;
    });
  }

  async function changePassword(currentPassword: string, newPassword: string) {
    return run("changePassword", async () => {
      status.value = await changeAccountPassword(currentPassword, newPassword);
      feedback.value = "密码修改成功，其他设备已退出登录。";
      return true;
    });
  }

  async function logout() {
    return run("logout", async () => {
      status.value = await logoutAccount();
      feedback.value = null;
      return true;
    });
  }

  function openDialog() {
    clearMessages();
    isDialogVisible.value = true;
  }

  function clearMessages() {
    error.value = null;
    feedback.value = null;
  }

  async function run<T>(action: AccountAction, operation: () => Promise<T>, silent = false): Promise<T | null> {
    activeAction.value = action;
    if (!silent) clearMessages();
    try {
      return await operation();
    } catch (operationError) {
      error.value = formatError(operationError, "账号操作失败，请稍后重试。");
      return null;
    } finally {
      activeAction.value = null;
    }
  }

  return {
    activeAction,
    changePassword,
    clearMessages,
    error,
    feedback,
    isDialogVisible,
    isLoading,
    loadStatus,
    login,
    logout,
    openDialog,
    redeem,
    refreshStatus,
    register,
    status,
    topBarText,
  };
}

function formatError(error: unknown, fallback: string) {
  if (error instanceof Error) return error.message;
  const message = String(error ?? "").trim();
  return message || fallback;
}
