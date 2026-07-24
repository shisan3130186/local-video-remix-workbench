export type MembershipState = "active" | "expired" | "none" | "disabled";

export interface SignedAccountPayload {
  schemaVersion: 1;
  userId: string;
  email: string;
  displayName: string;
  membershipStatus: MembershipState;
  expiresAt: number | null;
  issuedAt: number;
  offlineUntil: number;
  serverTime: number;
}

const CODE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

export function normalizeEmail(value: string): string {
  return value.trim().toLowerCase();
}

export function validateEmail(value: string): string {
  const email = normalizeEmail(value);
  if (email.length < 5 || email.length > 254 || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    throw new Error("请输入正确的邮箱地址。");
  }
  return email;
}

export function validatePassword(value: string): string {
  if (value.length < 8 || value.length > 128) {
    throw new Error("密码必须是8到128个字符。");
  }
  return value;
}

export function normalizeDisplayName(value: string, fallback: string): string {
  const normalized = value.trim();
  if (!normalized) return fallback;
  if (normalized.length > 24) throw new Error("昵称最多24个字符。");
  return normalized;
}

export function normalizeRedemptionCode(value: string): string {
  return value.toUpperCase().replace(/[^A-Z0-9]/g, "");
}

export function formatRedemptionCode(bytes: Uint8Array): string {
  let body = "";
  for (const byte of bytes) body += CODE_ALPHABET[byte % CODE_ALPHABET.length];
  return `SCUT-${body.slice(0, 4)}-${body.slice(4, 8)}-${body.slice(8, 12)}-${body.slice(12, 16)}`;
}

export function createRedemptionCode(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return formatRedemptionCode(bytes);
}

export function createRandomToken(byteLength = 32): string {
  const bytes = new Uint8Array(byteLength);
  crypto.getRandomValues(bytes);
  return bytesToBase64Url(bytes);
}

export async function sha256Hex(value: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

export async function hashPassword(password: string, saltBase64?: string, iterations = 100_000) {
  const salt = saltBase64 ? base64ToBytes(saltBase64) : crypto.getRandomValues(new Uint8Array(16));
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(password),
    "PBKDF2",
    false,
    ["deriveBits"],
  );
  const bits = await crypto.subtle.deriveBits(
    { name: "PBKDF2", hash: "SHA-256", salt: salt.slice().buffer as ArrayBuffer, iterations },
    key,
    256,
  );
  return { hash: bytesToBase64(new Uint8Array(bits)), salt: bytesToBase64(salt), iterations };
}

export function constantTimeEqual(left: string, right: string): boolean {
  const leftBytes = new TextEncoder().encode(left);
  const rightBytes = new TextEncoder().encode(right);
  let difference = leftBytes.length ^ rightBytes.length;
  const length = Math.max(leftBytes.length, rightBytes.length);
  for (let index = 0; index < length; index += 1) {
    difference |= (leftBytes[index] ?? 0) ^ (rightBytes[index] ?? 0);
  }
  return difference === 0;
}

export function resolveMembershipState(
  userStatus: string,
  membershipStatus: string | null,
  expiresAt: number | null,
  now: number,
): MembershipState {
  if (userStatus === "disabled" || membershipStatus === "disabled") return "disabled";
  if (expiresAt === null) return "none";
  return expiresAt <= now ? "expired" : "active";
}

export function buildSignedAccount(input: {
  userId: string;
  email: string;
  displayName: string;
  membershipStatus: MembershipState;
  expiresAt: number | null;
  now: number;
  graceHours: number;
}): SignedAccountPayload {
  const offlineUntil = input.membershipStatus === "active" && input.expiresAt !== null
    ? Math.min(input.expiresAt, input.now + input.graceHours * 60 * 60)
    : input.now;
  return {
    schemaVersion: 1,
    userId: input.userId,
    email: input.email,
    displayName: input.displayName,
    membershipStatus: input.membershipStatus,
    expiresAt: input.expiresAt,
    issuedAt: input.now,
    offlineUntil,
    serverTime: input.now,
  };
}

export async function signAccount(payload: SignedAccountPayload, privateKeyBase64: string): Promise<string> {
  const keyBytes = base64ToBytes(privateKeyBase64);
  const key = await crypto.subtle.importKey(
    "pkcs8",
    keyBytes.slice().buffer as ArrayBuffer,
    { name: "ECDSA", namedCurve: "P-256" },
    false,
    ["sign"],
  );
  const message = new TextEncoder().encode(JSON.stringify(payload));
  const signature = await crypto.subtle.sign({ name: "ECDSA", hash: "SHA-256" }, key, message);
  return bytesToBase64(new Uint8Array(signature));
}

export function safeDurationDays(value: unknown): number {
  const number = Number(value);
  if (!Number.isInteger(number) || number < 1 || number > 3650) {
    throw new Error("有效天数必须是1到3650之间的整数。");
  }
  return number;
}

function bytesToBase64(value: Uint8Array): string {
  let binary = "";
  for (const byte of value) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  const binary = atob(value.replace(/\s/g, ""));
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function bytesToBase64Url(value: Uint8Array): string {
  return bytesToBase64(value).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}
