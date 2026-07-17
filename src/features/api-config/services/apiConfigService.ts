import { invoke } from "@tauri-apps/api/core";
import type {
  ApiConfigInput,
  ApiConfigStatus,
  ApiCredentialKind,
} from "../types";

export function getApiConfigStatus() {
  return invoke<ApiConfigStatus>("get_api_config_status");
}

export function saveApiConfig(input: ApiConfigInput) {
  return invoke<ApiConfigStatus>("save_api_config", { input });
}

export function deleteApiCredential(kind: ApiCredentialKind) {
  return invoke<ApiConfigStatus>("delete_api_credential", { kind });
}
