import {
  buildSignedAccount,
  constantTimeEqual,
  createRandomToken,
  createRedemptionCode,
  hashPassword,
  normalizeDisplayName,
  normalizeRedemptionCode,
  resolveMembershipState,
  safeDurationDays,
  sha256Hex,
  signAccount,
  validateEmail,
  validatePassword,
} from "./shared.ts";

interface Env {
  DB: D1Database;
  ADMIN_TOKEN: string;
  SIGNING_PRIVATE_KEY_PKCS8_BASE64: string;
  OFFLINE_GRACE_HOURS?: string;
}

interface UserRow {
  id: string;
  email: string;
  display_name: string;
  password_hash: string;
  password_salt: string;
  password_iterations: number;
  status: "active" | "disabled";
}

interface AccountRow {
  id: string;
  email: string;
  display_name: string;
  user_status: "active" | "disabled";
  membership_status: "active" | "disabled" | null;
  expires_at: number | null;
  bound_device_hash: string | null;
  rebind_month: string | null;
  rebind_count: number;
  current_device_hash?: string | null;
}

interface SessionAccountRow extends AccountRow {
  session_id: string;
  current_device_hash: string | null;
}

interface RedemptionCodeRow {
  id: string;
  duration_days: number;
  status: "unused" | "redeemed" | "disabled";
}

interface AuthInput {
  email?: string;
  password?: string;
  displayName?: string;
  appVersion?: string;
  deviceId?: string;
}

class HttpError extends Error {
  constructor(public status: number, public code: string, message: string) {
    super(message);
  }
}

const JSON_HEADERS = {
  "content-type": "application/json; charset=utf-8",
  "cache-control": "no-store",
};

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    try {
      if (request.method === "GET" && url.pathname === "/health") {
        return json({ ok: true, service: "smartcut-membership", time: unixNow() });
      }
      if (request.method === "POST" && url.pathname === "/v1/auth/register") return await register(request, env);
      if (request.method === "POST" && url.pathname === "/v1/auth/login") return await login(request, env);
      if (request.method === "POST" && url.pathname === "/v1/auth/logout") return await logout(request, env);
      if (request.method === "GET" && url.pathname === "/v1/account/me") return await getAccount(request, env);
      if (request.method === "POST" && url.pathname === "/v1/account/redeem") return await redeem(request, env);
      if (request.method === "POST" && url.pathname === "/v1/account/change-password") {
        return await changePassword(request, env);
      }
      if (url.pathname === "/v1/admin/codes" && request.method === "POST") {
        requireAdmin(request, env);
        return await createCode(request, env);
      }
      if (url.pathname === "/v1/admin/codes" && request.method === "GET") {
        requireAdmin(request, env);
        return await listCodes(env);
      }
      const codeMatch = url.pathname.match(/^\/v1\/admin\/codes\/([a-zA-Z0-9-]+)$/);
      if (codeMatch && request.method === "PATCH") {
        requireAdmin(request, env);
        return await updateCode(request, env, codeMatch[1]);
      }
      throw new HttpError(404, "not_found", "没有找到这个接口。");
    } catch (error) {
      if (error instanceof HttpError) return errorResponse(error.status, error.code, error.message);
      const message = error instanceof Error ? error.message : "服务暂时不可用。";
      return errorResponse(400, "bad_request", message);
    }
  },
};

async function register(request: Request, env: Env): Promise<Response> {
  const input = await readJson<AuthInput>(request);
  const email = validateEmail(String(input.email ?? ""));
  const password = validatePassword(String(input.password ?? ""));
  const exists = await env.DB.prepare("SELECT id FROM users WHERE email = ?1").bind(email).first<{ id: string }>();
  if (exists) throw new HttpError(409, "email_exists", "这个邮箱已经注册，请直接登录。" );

  const now = unixNow();
  const currentDeviceHash = await hashDeviceId(input.deviceId);
  const userId = crypto.randomUUID();
  const displayName = normalizeDisplayName(String(input.displayName ?? ""), `智剪用户${userId.slice(-4)}`);
  const passwordData = await hashPassword(password);
  await env.DB.prepare(
    `INSERT INTO users
      (id, email, display_name, password_hash, password_salt, password_iterations, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)`,
  ).bind(userId, email, displayName, passwordData.hash, passwordData.salt, passwordData.iterations, now).run();
  const sessionToken = await createSession(env, userId, now, currentDeviceHash);
  await recordEvent(env, userId, "register", "success", input.appVersion);
  const account = await loadAccount(env, userId);
  return await signedAccountResponse(env, { ...account, current_device_hash: currentDeviceHash }, sessionToken, "注册成功。", 201);
}

async function login(request: Request, env: Env): Promise<Response> {
  const input = await readJson<AuthInput>(request);
  const email = validateEmail(String(input.email ?? ""));
  const password = validatePassword(String(input.password ?? ""));
  const user = await env.DB.prepare(
    "SELECT id, email, display_name, password_hash, password_salt, password_iterations, status FROM users WHERE email = ?1",
  ).bind(email).first<UserRow>();
  if (!user) throw new HttpError(401, "invalid_credentials", "邮箱或密码不正确。" );
  const candidate = await hashPassword(password, user.password_salt, user.password_iterations);
  if (!constantTimeEqual(candidate.hash, user.password_hash)) {
    await recordEvent(env, user.id, "login", "invalid_password", input.appVersion);
    throw new HttpError(401, "invalid_credentials", "邮箱或密码不正确。" );
  }
  if (user.status === "disabled") throw new HttpError(403, "account_disabled", "此账号已被停用。" );
  const now = unixNow();
  const currentDeviceHash = await hashDeviceId(input.deviceId);
  const sessionToken = await createSession(env, user.id, now, currentDeviceHash);
  await recordEvent(env, user.id, "login", "success", input.appVersion);
  return await signedAccountResponse(env, { ...(await loadAccount(env, user.id)), current_device_hash: currentDeviceHash }, sessionToken, "登录成功。");
}

async function logout(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  await env.DB.prepare("DELETE FROM sessions WHERE id = ?1").bind(session.session_id).run();
  await recordEvent(env, session.id, "logout", "success", "");
  return json({ ok: true, message: "已退出登录。" });
}

async function getAccount(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  return await signedAccountResponse(env, session, null, "账号状态正常。" , 200, session.current_device_hash);
}

async function redeem(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<{ redemptionCode?: string; appVersion?: string; allowDeviceRebind?: boolean }>(request);
  const normalized = normalizeRedemptionCode(String(body.redemptionCode ?? ""));
  if (normalized.length !== 20 || !normalized.startsWith("SCUT")) {
    throw new HttpError(400, "invalid_code", "兑换码格式不正确。" );
  }
  const codeHash = await sha256Hex(normalized);
  const code = await env.DB.prepare(
    "SELECT id, duration_days, status FROM redemption_codes WHERE code_hash = ?1",
  ).bind(codeHash).first<RedemptionCodeRow>();
  if (!code) throw new HttpError(404, "invalid_code", "兑换码不存在或输入有误。" );
  if (code.status === "redeemed") throw new HttpError(409, "code_redeemed", "这个兑换码已经使用过了。" );
  if (code.status === "disabled") throw new HttpError(403, "code_disabled", "这个兑换码已被停用。" );

  const now = unixNow();
  const currentDeviceHash = session.current_device_hash;
  if (!currentDeviceHash) throw new HttpError(400, "device_required", "无法识别当前设备，请重新启动软件后重试。" );
  const membership = await env.DB.prepare(
    "SELECT expires_at, bound_device_hash, rebind_month, rebind_count FROM memberships WHERE user_id = ?1",
  ).bind(session.id).first<{ expires_at: number; bound_device_hash: string | null; rebind_month: string | null; rebind_count: number }>();
  const month = monthKey(now);
  let boundDeviceHash = membership?.bound_device_hash ?? currentDeviceHash;
  let rebindMonth = membership?.rebind_month ?? month;
  let rebindCount = membership?.rebind_count ?? 0;
  if (membership?.bound_device_hash && membership.bound_device_hash !== currentDeviceHash) {
    if (!body.allowDeviceRebind) throw new HttpError(409, "device_rebind_required", "兑换后会员账号将绑定到此设备，如需在其他设备使用需进行换绑（每月限2次），确定要继续吗？" );
    if (rebindMonth !== month) rebindCount = 0;
    if (rebindCount >= 2) throw new HttpError(429, "device_rebind_limit", "本月换绑次数已用完，请下月再试。" );
    boundDeviceHash = currentDeviceHash;
    rebindMonth = month;
    rebindCount += 1;
  }
  const claimed = await env.DB.prepare(
    `UPDATE redemption_codes
     SET status = 'redeemed', redeemed_by_user_id = ?1, redeemed_at = ?2, updated_at = ?2
     WHERE id = ?3 AND status = 'unused'`,
  ).bind(session.id, now, code.id).run();
  if (Number(claimed.meta.changes ?? 0) !== 1) {
    throw new HttpError(409, "code_redeemed", "这个兑换码刚刚已被使用。" );
  }

  const baseTime = Math.max(now, membership?.expires_at ?? now);
  const expiresAt = baseTime + code.duration_days * 24 * 60 * 60;
  await env.DB.prepare(
    `INSERT INTO memberships (user_id, status, expires_at, bound_device_hash, rebind_month, rebind_count, updated_at)
     VALUES (?1, 'active', ?2, ?3, ?4, ?5, ?6)
     ON CONFLICT(user_id) DO UPDATE SET status = 'active', expires_at = excluded.expires_at,
       bound_device_hash = excluded.bound_device_hash, rebind_month = excluded.rebind_month,
       rebind_count = excluded.rebind_count, updated_at = excluded.updated_at`,
  ).bind(session.id, expiresAt, boundDeviceHash, rebindMonth, rebindCount, now).run();
  await recordEvent(env, session.id, "redeem", `success_${code.duration_days}_days`, body.appVersion);
  return await signedAccountResponse(env, { ...(await loadAccount(env, session.id)), current_device_hash: currentDeviceHash }, null, `兑换成功，会员已增加${code.duration_days}天。`);
}

async function changePassword(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<{ currentPassword?: string; newPassword?: string; appVersion?: string }>(request);
  const currentPassword = validatePassword(String(body.currentPassword ?? ""));
  const newPassword = validatePassword(String(body.newPassword ?? ""));
  if (currentPassword === newPassword) throw new HttpError(400, "same_password", "新密码不能和当前密码相同。" );
  const user = await env.DB.prepare(
    "SELECT id, email, display_name, password_hash, password_salt, password_iterations, status FROM users WHERE id = ?1",
  ).bind(session.id).first<UserRow>();
  if (!user) throw new HttpError(401, "session_invalid", "登录状态已失效，请重新登录。" );
  const candidate = await hashPassword(currentPassword, user.password_salt, user.password_iterations);
  if (!constantTimeEqual(candidate.hash, user.password_hash)) {
    throw new HttpError(401, "invalid_password", "当前密码不正确。" );
  }
  const replacement = await hashPassword(newPassword);
  const now = unixNow();
  await env.DB.prepare(
    "UPDATE users SET password_hash = ?1, password_salt = ?2, password_iterations = ?3, updated_at = ?4 WHERE id = ?5",
  ).bind(replacement.hash, replacement.salt, replacement.iterations, now, session.id).run();
  await env.DB.prepare("DELETE FROM sessions WHERE user_id = ?1 AND id <> ?2")
    .bind(session.id, session.session_id).run();
  await recordEvent(env, session.id, "change_password", "success", body.appVersion);
  return await signedAccountResponse(env, { ...(await loadAccount(env, session.id)), current_device_hash: session.current_device_hash }, null, "密码修改成功，其他设备已退出登录。" );
}

async function createCode(request: Request, env: Env): Promise<Response> {
  const body = await readJson<Record<string, unknown>>(request);
  const durationDays = safeDurationDays(body.durationDays);
  const label = String(body.label ?? "").trim().slice(0, 80);
  const redemptionCode = createRedemptionCode();
  const codeHash = await sha256Hex(normalizeRedemptionCode(redemptionCode));
  const id = crypto.randomUUID();
  const now = unixNow();
  await env.DB.prepare(
    "INSERT INTO redemption_codes (id, code_hash, duration_days, label, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
  ).bind(id, codeHash, durationDays, label, now).run();
  return json({ ok: true, codeId: id, redemptionCode, durationDays, label }, 201);
}

async function listCodes(env: Env): Promise<Response> {
  const result = await env.DB.prepare(
    `SELECT c.id, c.duration_days, c.label, c.status, c.redeemed_at, c.created_at,
      u.email AS redeemed_by_email
     FROM redemption_codes c
     LEFT JOIN users u ON u.id = c.redeemed_by_user_id
     ORDER BY c.created_at DESC LIMIT 500`,
  ).all<Record<string, unknown>>();
  return json({ ok: true, codes: result.results ?? [] });
}

async function updateCode(request: Request, env: Env, codeId: string): Promise<Response> {
  const body = await readJson<{ action?: string }>(request);
  const action = String(body.action ?? "");
  if (action !== "disable" && action !== "enable") throw new HttpError(400, "bad_action", "不支持的管理操作。" );
  const nextStatus = action === "disable" ? "disabled" : "unused";
  const result = await env.DB.prepare(
    "UPDATE redemption_codes SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status <> 'redeemed'",
  ).bind(nextStatus, unixNow(), codeId).run();
  if (Number(result.meta.changes ?? 0) !== 1) {
    throw new HttpError(409, "code_not_editable", "兑换码不存在或已经使用，无法修改。" );
  }
  return json({ ok: true, codeId, status: nextStatus });
}

async function createSession(env: Env, userId: string, now: number, deviceHash: string): Promise<string> {
  const token = createRandomToken();
  const tokenHash = await sha256Hex(token);
  const expiresAt = now + 30 * 24 * 60 * 60;
  await env.DB.prepare(
    "INSERT INTO sessions (id, user_id, token_hash, expires_at, created_at, last_seen_at, device_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
  ).bind(crypto.randomUUID(), userId, tokenHash, expiresAt, now, deviceHash).run();
  return token;
}

async function requireSession(request: Request, env: Env): Promise<SessionAccountRow> {
  const token = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "").trim() ?? "";
  if (token.length < 32 || token.length > 256) throw new HttpError(401, "session_required", "请先登录账号。" );
  const tokenHash = await sha256Hex(token);
  const now = unixNow();
  const session = await env.DB.prepare(
    `SELECT s.id AS session_id, u.id, u.email, u.display_name, u.status AS user_status,
      m.status AS membership_status, m.expires_at, m.bound_device_hash, m.rebind_month, m.rebind_count,
      s.device_hash AS current_device_hash
     FROM sessions s
     JOIN users u ON u.id = s.user_id
     LEFT JOIN memberships m ON m.user_id = u.id
     WHERE s.token_hash = ?1 AND s.expires_at > ?2`,
  ).bind(tokenHash, now).first<SessionAccountRow>();
  if (!session) throw new HttpError(401, "session_invalid", "登录状态已失效，请重新登录。" );
  await env.DB.prepare("UPDATE sessions SET last_seen_at = ?1 WHERE id = ?2").bind(now, session.session_id).run();
  return session;
}

async function loadAccount(env: Env, userId: string): Promise<AccountRow> {
  const account = await env.DB.prepare(
    `SELECT u.id, u.email, u.display_name, u.status AS user_status,
      m.status AS membership_status, m.expires_at, m.bound_device_hash, m.rebind_month, m.rebind_count
     FROM users u LEFT JOIN memberships m ON m.user_id = u.id WHERE u.id = ?1`,
  ).bind(userId).first<AccountRow>();
  if (!account) throw new HttpError(404, "account_not_found", "账号不存在。" );
  return account;
}

async function signedAccountResponse(
  env: Env,
  account: AccountRow,
  sessionToken: string | null,
  message: string,
  status = 200,
  currentDeviceHash: string | null = account.current_device_hash ?? null,
): Promise<Response> {
  const now = unixNow();
  const membershipStatus = resolveMembershipState(
    account.user_status,
    account.membership_status,
    account.expires_at,
    now,
  );
  const graceHours = Math.min(168, Math.max(1, Number(env.OFFLINE_GRACE_HOURS ?? 72) || 72));
  const currentMonth = monthKey(now);
  const deviceBound = Boolean(account.bound_device_hash);
  const deviceMatch = !deviceBound || Boolean(currentDeviceHash && currentDeviceHash === account.bound_device_hash);
  const rebindsUsed = account.rebind_month === currentMonth ? account.rebind_count : 0;
  const payload = buildSignedAccount({
    userId: account.id,
    email: account.email,
    displayName: account.display_name,
    membershipStatus,
    expiresAt: account.expires_at,
    now,
    graceHours,
    deviceBound,
    deviceMatch,
    rebindsRemaining: Math.max(0, 2 - rebindsUsed),
  });
  const signature = await signAccount(payload, env.SIGNING_PRIVATE_KEY_PKCS8_BASE64);
  return json({ ok: true, message, sessionToken, account: payload, signature }, status);
}

async function recordEvent(
  env: Env,
  userId: string | null,
  action: string,
  outcome: string,
  appVersion: unknown,
) {
  await env.DB.prepare(
    "INSERT INTO membership_events (user_id, action, outcome, app_version, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
  ).bind(userId, action, outcome, String(appVersion ?? "").trim().slice(0, 32), unixNow()).run();
}

function requireAdmin(request: Request, env: Env) {
  const provided = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "") ?? "";
  if (!env.ADMIN_TOKEN || !constantTimeEqual(provided, env.ADMIN_TOKEN)) {
    throw new HttpError(401, "unauthorized", "管理凭证无效。" );
  }
}

async function readJson<T>(request: Request): Promise<T> {
  if (!(request.headers.get("content-type") ?? "").includes("application/json")) {
    throw new HttpError(415, "invalid_content_type", "请求格式必须是JSON。" );
  }
  return (await request.json()) as T;
}

function unixNow(): number {
  return Math.floor(Date.now() / 1000);
}

async function hashDeviceId(value: unknown): Promise<string> {
  const normalized = String(value ?? "").trim();
  if (!normalized || normalized.length > 128) throw new HttpError(400, "device_required", "无法识别当前设备，请重新启动软件后重试。" );
  return sha256Hex(normalized);
}

function monthKey(now: number): string {
  return new Date(now * 1000).toISOString().slice(0, 7);
}

function json(value: unknown, status = 200): Response {
  return new Response(JSON.stringify(value), { status, headers: JSON_HEADERS });
}

function errorResponse(status: number, code: string, message: string): Response {
  return json({ ok: false, code, message }, status);
}
