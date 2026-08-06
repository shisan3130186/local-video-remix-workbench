export interface AccountStatus {
  signedIn: boolean;
  memberActive: boolean;
  membershipState: "signed_out" | "active" | "expired" | "none" | "disabled" | "invalid" | "verification_required";
  accessMode: "signed_out" | "online" | "offline" | "cached" | "blocked";
  serviceConfigured: boolean;
  email: string | null;
  displayName: string | null;
  expiresAt: number | null;
  offlineUntil: number | null;
  lastValidatedAt: number | null;
  deviceBound: boolean;
  deviceMatch: boolean;
  rebindsRemaining: number;
  message: string;
}

export type AccountAction = "register" | "login" | "refresh" | "redeem" | "changePassword" | "logout";
