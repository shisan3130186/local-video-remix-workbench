import { invoke } from "@tauri-apps/api/core";
import type { AccountStatus } from "../types";

export function getAccountStatus() {
  return invoke<AccountStatus>("get_account_status");
}

export function registerAccount(email: string, password: string, displayName: string) {
  return invoke<AccountStatus>("register_account", { email, password, displayName });
}

export function loginAccount(email: string, password: string) {
  return invoke<AccountStatus>("login_account", { email, password });
}

export function refreshAccount() {
  return invoke<AccountStatus>("refresh_account");
}

export function redeemMembership(redemptionCode: string, allowDeviceRebind: boolean) {
  return invoke<AccountStatus>("redeem_membership", { redemptionCode, allowDeviceRebind });
}

export function changeAccountPassword(currentPassword: string, newPassword: string) {
  return invoke<AccountStatus>("change_account_password", { currentPassword, newPassword });
}

export function logoutAccount() {
  return invoke<AccountStatus>("logout_account");
}
