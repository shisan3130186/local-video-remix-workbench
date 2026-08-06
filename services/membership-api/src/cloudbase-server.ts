import cloudbase from "@cloudbase/node-sdk";
import express, { type NextFunction, type Request, type Response } from "express";
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
} from "./shared.js";

type MembershipStatus = "active" | "disabled";

interface UserDocument {
  _id: string;
  id: string;
  email: string;
  display_name: string;
  password_hash: string;
  password_salt: string;
  password_iterations: number;
  status: "active" | "disabled";
  auth_version: number;
  created_at: number;
  updated_at: number;
}

interface SessionDocument {
  _id: string;
  user_doc_id: string;
  user_id: string;
  auth_version: number;
  expires_at: number;
  created_at: number;
  last_seen_at: number;
  device_hash?: string;
}

interface MembershipDocument {
  _id: string;
  user_id: string;
  status: MembershipStatus;
  expires_at: number;
  updated_at: number;
  bound_device_hash?: string | null;
  rebind_month?: string | null;
  rebind_count?: number;
}

interface RedemptionCodeDocument {
  _id: string;
  id: string;
  duration_days: number;
  label: string;
  status: "unused" | "redeemed" | "disabled";
  redeemed_by_user_id: string | null;
  redeemed_by_email: string | null;
  redeemed_at: number | null;
  created_at: number;
  updated_at: number;
}

interface AccountRow {
  id: string;
  email: string;
  display_name: string;
  user_status: "active" | "disabled";
  membership_status: MembershipStatus | null;
  expires_at: number | null;
  bound_device_hash: string | null;
  rebind_month: string | null;
  rebind_count: number;
  current_device_hash?: string | null;
}

interface SessionAccountRow extends AccountRow {
  session_doc_id: string;
  user_doc_id: string;
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

const app = cloudbase.init({ env: cloudbase.SYMBOL_CURRENT_ENV });
// CloudBase Node SDK 的数据库类型没有完整暴露事务泛型，这里集中在适配层使用。
const db: any = app.database();
const server = express();

server.disable("x-powered-by");
server.use(express.json({ limit: "32kb", type: "application/json" }));
server.use((request, _response, next) => {
  const gatewayPrefix = "/smartcut-membership";
  if (request.url === gatewayPrefix) request.url = "/";
  if (request.url.startsWith(`${gatewayPrefix}/`)) request.url = request.url.slice(gatewayPrefix.length);
  next();
});
server.use("/v1", (_request, _response, next) => {
  ensureDatabaseReady().then(() => next()).catch(next);
});

server.get("/health", (_request, response) => {
  response.set("cache-control", "no-store").json({
    ok: true,
    service: "smartcut-membership",
    platform: "cloudbase",
    time: unixNow(),
  });
});

server.post("/v1/auth/register", asyncRoute(register));
server.post("/v1/auth/login", asyncRoute(login));
server.post("/v1/auth/logout", asyncRoute(logout));
server.get("/v1/account/me", asyncRoute(getAccount));
server.post("/v1/account/redeem", asyncRoute(redeem));
server.post("/v1/account/change-password", asyncRoute(changePassword));
server.post("/v1/admin/codes", asyncRoute(async (request, response) => {
  requireAdmin(request);
  await createCode(request, response);
}));
server.get("/v1/admin/codes", asyncRoute(async (request, response) => {
  requireAdmin(request);
  await listCodes(response);
}));
server.patch("/v1/admin/codes/:codeId", asyncRoute(async (request, response) => {
  requireAdmin(request);
  await updateCode(request, response, String(request.params.codeId));
}));

server.use((_request, _response, next) => next(new HttpError(404, "not_found", "没有找到这个接口。")));
server.use((error: unknown, _request: Request, response: Response, _next: NextFunction) => {
  if (error instanceof HttpError) {
    response.status(error.status).json({ ok: false, code: error.code, message: error.message });
    return;
  }
  console.error("membership request failed", error);
  response.status(500).json({ ok: false, code: "service_unavailable", message: "会员服务暂时不可用，请稍后重试。" });
});

const port = Number(process.env.PORT ?? 9000);
server.listen(port, "0.0.0.0", () => {
  console.log(`SmartCut membership API listening on ${port}`);
});

let databaseReadyPromise: Promise<void> | null = null;

function ensureDatabaseReady(): Promise<void> {
  if (databaseReadyPromise) return databaseReadyPromise;
  databaseReadyPromise = (async () => {
    const collections = ["users", "sessions", "memberships", "redemption_codes", "membership_events"];
    for (const collectionName of collections) {
      await db.collection(collectionName).doc("__schema__").set({
        system: true,
        schema_version: 1,
      });
    }
  })().catch((error) => {
    databaseReadyPromise = null;
    throw error;
  });
  return databaseReadyPromise;
}

async function register(request: Request, response: Response): Promise<void> {
  const input = request.body as AuthInput;
  const email = validateInput(() => validateEmail(String(input.email ?? "")));
  const password = validateInput(() => validatePassword(String(input.password ?? "")));
  const userDocId = await sha256Hex(email);
  if (await getDocument<UserDocument>("users", userDocId)) {
    throw new HttpError(409, "email_exists", "这个邮箱已经注册，请直接登录。");
  }

  const now = unixNow();
  const currentDeviceHash = await hashDeviceId(input.deviceId);
  const userId = crypto.randomUUID();
  const displayName = validateInput(() => normalizeDisplayName(String(input.displayName ?? ""), `智剪用户${userId.slice(-4)}`));
  const passwordData = await hashPassword(password);
  const user: UserDocument = {
    _id: userDocId,
    id: userId,
    email,
    display_name: displayName,
    password_hash: passwordData.hash,
    password_salt: passwordData.salt,
    password_iterations: passwordData.iterations,
    status: "active",
    auth_version: 1,
    created_at: now,
    updated_at: now,
  };

  try {
    await db.collection("users").add(user);
  } catch (error) {
    if (isDuplicateDocumentError(error)) {
      throw new HttpError(409, "email_exists", "这个邮箱已经注册，请直接登录。");
    }
    throw error;
  }

  const sessionToken = await createSession(user, now, currentDeviceHash);
  await recordEvent(user.id, "register", "success", input.appVersion);
  await sendSignedAccount(response, { ...(await loadAccount(userDocId)), current_device_hash: currentDeviceHash }, sessionToken, "注册成功。", 201);
}

async function login(request: Request, response: Response): Promise<void> {
  const input = request.body as AuthInput;
  const email = validateInput(() => validateEmail(String(input.email ?? "")));
  const password = validateInput(() => validatePassword(String(input.password ?? "")));
  const userDocId = await sha256Hex(email);
  const user = await getDocument<UserDocument>("users", userDocId);
  if (!user) throw new HttpError(401, "invalid_credentials", "邮箱或密码不正确。");
  const candidate = await hashPassword(password, user.password_salt, user.password_iterations);
  if (!constantTimeEqual(candidate.hash, user.password_hash)) {
    await recordEvent(user.id, "login", "invalid_password", input.appVersion);
    throw new HttpError(401, "invalid_credentials", "邮箱或密码不正确。");
  }
  if (user.status === "disabled") throw new HttpError(403, "account_disabled", "此账号已被停用。");

  const now = unixNow();
  const currentDeviceHash = await hashDeviceId(input.deviceId);
  const sessionToken = await createSession(user, now, currentDeviceHash);
  await recordEvent(user.id, "login", "success", input.appVersion);
  await sendSignedAccount(response, { ...(await loadAccount(userDocId)), current_device_hash: currentDeviceHash }, sessionToken, "登录成功。");
}

async function logout(request: Request, response: Response): Promise<void> {
  const session = await requireSession(request);
  await db.collection("sessions").doc(session.session_doc_id).remove();
  await recordEvent(session.id, "logout", "success", "");
  response.json({ ok: true, message: "已退出登录。" });
}

async function getAccount(request: Request, response: Response): Promise<void> {
  const session = await requireSession(request);
  await sendSignedAccount(response, session, null, "账号状态正常。", 200, session.current_device_hash);
}

async function redeem(request: Request, response: Response): Promise<void> {
  const session = await requireSession(request);
  const body = request.body as { redemptionCode?: string; appVersion?: string; allowDeviceRebind?: boolean };
  const normalized = normalizeRedemptionCode(String(body.redemptionCode ?? ""));
  if (normalized.length !== 20 || !normalized.startsWith("SCUT")) {
    throw new HttpError(400, "invalid_code", "兑换码格式不正确。");
  }
  const codeHash = await sha256Hex(normalized);
  const now = unixNow();
  const currentDeviceHash = session.current_device_hash;
  if (!currentDeviceHash) throw new HttpError(400, "device_required", "无法识别当前设备，请重新启动软件后重试。");
  const month = monthKey(now);

  const durationDays = Number(await db.runTransaction(async (transaction: any) => {
    const codeReference = transaction.collection("redemption_codes").doc(codeHash);
    const membershipReference = transaction.collection("memberships").doc(session.user_doc_id);
    const code = normalizeDocument<RedemptionCodeDocument>((await codeReference.get()).data);
    if (!code) throw new HttpError(404, "invalid_code", "兑换码不存在或输入有误。");
    if (code.status === "redeemed") throw new HttpError(409, "code_redeemed", "这个兑换码已经使用过了。");
    if (code.status === "disabled") throw new HttpError(403, "code_disabled", "这个兑换码已被停用。");

    const membership = normalizeDocument<MembershipDocument>((await membershipReference.get()).data);
    let boundDeviceHash = membership?.bound_device_hash ?? currentDeviceHash;
    let rebindMonth = membership?.rebind_month ?? month;
    let rebindCount = membership?.rebind_count ?? 0;
    if (membership?.bound_device_hash && membership.bound_device_hash !== currentDeviceHash) {
      if (!body.allowDeviceRebind) throw new HttpError(409, "device_rebind_required", "兑换后会员账号将绑定到此设备，如需在其他设备使用需进行换绑（每月限2次），确定要继续吗？");
      if (rebindMonth !== month) rebindCount = 0;
      if (rebindCount >= 2) throw new HttpError(429, "device_rebind_limit", "本月换绑次数已用完，请下月再试。");
      boundDeviceHash = currentDeviceHash;
      rebindMonth = month;
      rebindCount += 1;
    }
    const baseTime = Math.max(now, membership?.expires_at ?? now);
    const expiresAt = baseTime + code.duration_days * 24 * 60 * 60;

    await codeReference.update({
      status: "redeemed",
      redeemed_by_user_id: session.id,
      redeemed_by_email: session.email,
      redeemed_at: now,
      updated_at: now,
    });
    await membershipReference.set({
      user_id: session.id,
      status: "active",
      expires_at: expiresAt,
      bound_device_hash: boundDeviceHash,
      rebind_month: rebindMonth,
      rebind_count: rebindCount,
      updated_at: now,
    });
    return code.duration_days;
  }));

  await recordEvent(session.id, "redeem", `success_${durationDays}_days`, body.appVersion);
  await sendSignedAccount(
    response,
    await loadAccount(session.user_doc_id),
    null,
    `兑换成功，会员已增加${durationDays}天。`,
    200,
    currentDeviceHash,
  );
}

async function changePassword(request: Request, response: Response): Promise<void> {
  const session = await requireSession(request);
  const body = request.body as { currentPassword?: string; newPassword?: string; appVersion?: string };
  const currentPassword = validateInput(() => validatePassword(String(body.currentPassword ?? "")));
  const newPassword = validateInput(() => validatePassword(String(body.newPassword ?? "")));
  if (currentPassword === newPassword) throw new HttpError(400, "same_password", "新密码不能和当前密码相同。");

  const user = await getDocument<UserDocument>("users", session.user_doc_id);
  if (!user) throw new HttpError(401, "session_invalid", "登录状态已失效，请重新登录。");
  const candidate = await hashPassword(currentPassword, user.password_salt, user.password_iterations);
  if (!constantTimeEqual(candidate.hash, user.password_hash)) {
    throw new HttpError(401, "invalid_password", "当前密码不正确。");
  }

  const replacement = await hashPassword(newPassword);
  const now = unixNow();
  const nextAuthVersion = user.auth_version + 1;
  await db.collection("users").doc(session.user_doc_id).update({
    password_hash: replacement.hash,
    password_salt: replacement.salt,
    password_iterations: replacement.iterations,
    auth_version: nextAuthVersion,
    updated_at: now,
  });
  await db.collection("sessions").doc(session.session_doc_id).update({
    auth_version: nextAuthVersion,
    last_seen_at: now,
  });
  await recordEvent(session.id, "change_password", "success", body.appVersion);
  await sendSignedAccount(response, { ...(await loadAccount(session.user_doc_id)), current_device_hash: session.current_device_hash }, null, "密码修改成功，其他设备已退出登录。");
}

async function createCode(request: Request, response: Response): Promise<void> {
  const body = request.body as Record<string, unknown>;
  const durationDays = validateInput(() => safeDurationDays(body.durationDays));
  const label = String(body.label ?? "").trim().slice(0, 80);
  const redemptionCode = createRedemptionCode();
  const codeHash = await sha256Hex(normalizeRedemptionCode(redemptionCode));
  const now = unixNow();
  const code: RedemptionCodeDocument = {
    _id: codeHash,
    id: codeHash,
    duration_days: durationDays,
    label,
    status: "unused",
    redeemed_by_user_id: null,
    redeemed_by_email: null,
    redeemed_at: null,
    created_at: now,
    updated_at: now,
  };
  await db.collection("redemption_codes").add(code);
  response.status(201).json({ ok: true, codeId: code.id, redemptionCode, durationDays, label });
}

async function listCodes(response: Response): Promise<void> {
  const codes: RedemptionCodeDocument[] = [];
  for (let offset = 0; offset < 500; offset += 100) {
    const result = await db.collection("redemption_codes").skip(offset).limit(100).get();
    const page = (result.data ?? []).filter((value: RedemptionCodeDocument) => value.status);
    codes.push(...page);
    if (page.length < 100) break;
  }
  codes.sort((left, right) => right.created_at - left.created_at);
  response.json({
    ok: true,
    codes: codes.map((code) => ({
      id: code.id,
      duration_days: code.duration_days,
      label: code.label,
      status: code.status,
      redeemed_at: code.redeemed_at,
      created_at: code.created_at,
      redeemed_by_email: code.redeemed_by_email,
    })),
  });
}

async function updateCode(request: Request, response: Response, codeId: string): Promise<void> {
  const body = request.body as { action?: string };
  const action = String(body.action ?? "");
  if (action !== "disable" && action !== "enable") {
    throw new HttpError(400, "bad_action", "不支持的管理操作。");
  }
  const code = await getDocument<RedemptionCodeDocument>("redemption_codes", codeId);
  if (!code || code.status === "redeemed") {
    throw new HttpError(409, "code_not_editable", "兑换码不存在或已经使用，无法修改。");
  }
  const nextStatus = action === "disable" ? "disabled" : "unused";
  await db.collection("redemption_codes").doc(codeId).update({
    status: nextStatus,
    updated_at: unixNow(),
  });
  response.json({ ok: true, codeId, status: nextStatus });
}

async function createSession(user: UserDocument, now: number, deviceHash: string): Promise<string> {
  const token = createRandomToken();
  const tokenHash = await sha256Hex(token);
  const session: SessionDocument = {
    _id: tokenHash,
    user_doc_id: user._id,
    user_id: user.id,
    auth_version: user.auth_version,
    expires_at: now + 30 * 24 * 60 * 60,
    created_at: now,
    last_seen_at: now,
    device_hash: deviceHash,
  };
  await db.collection("sessions").add(session);
  return token;
}

async function requireSession(request: Request): Promise<SessionAccountRow> {
  const token = request.get("authorization")?.replace(/^Bearer\s+/i, "").trim() ?? "";
  if (token.length < 32 || token.length > 256) throw new HttpError(401, "session_required", "请先登录账号。");
  const sessionDocId = await sha256Hex(token);
  const session = await getDocument<SessionDocument>("sessions", sessionDocId);
  const now = unixNow();
  if (!session || session.expires_at <= now) {
    if (session) await db.collection("sessions").doc(sessionDocId).remove();
    throw new HttpError(401, "session_invalid", "登录状态已失效，请重新登录。");
  }

  const user = await getDocument<UserDocument>("users", session.user_doc_id);
  if (!user || user.id !== session.user_id || user.auth_version !== session.auth_version) {
    await db.collection("sessions").doc(sessionDocId).remove();
    throw new HttpError(401, "session_invalid", "登录状态已失效，请重新登录。");
  }
  if (user.status === "disabled") throw new HttpError(403, "account_disabled", "此账号已被停用。");

  await db.collection("sessions").doc(sessionDocId).update({ last_seen_at: now });
  return {
    ...(await accountFromUser(user)),
    session_doc_id: sessionDocId,
    user_doc_id: user._id,
    current_device_hash: session.device_hash ?? null,
  };
}

async function loadAccount(userDocId: string): Promise<AccountRow> {
  const user = await getDocument<UserDocument>("users", userDocId);
  if (!user) throw new HttpError(404, "account_not_found", "账号不存在。");
  return accountFromUser(user);
}

async function accountFromUser(user: UserDocument): Promise<AccountRow> {
  const membership = await getDocument<MembershipDocument>("memberships", user._id);
  return {
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    user_status: user.status,
    membership_status: membership?.status ?? null,
    expires_at: membership?.expires_at ?? null,
    bound_device_hash: membership?.bound_device_hash ?? null,
    rebind_month: membership?.rebind_month ?? null,
    rebind_count: membership?.rebind_count ?? 0,
  };
}

async function sendSignedAccount(
  response: Response,
  account: AccountRow,
  sessionToken: string | null,
  message: string,
  status = 200,
  currentDeviceHash: string | null = account.current_device_hash ?? null,
): Promise<void> {
  const signingKey = requiredEnvironment("SIGNING_PRIVATE_KEY_PKCS8_BASE64");
  const now = unixNow();
  const membershipStatus = resolveMembershipState(
    account.user_status,
    account.membership_status,
    account.expires_at,
    now,
  );
  const graceHours = Math.min(168, Math.max(1, Number(process.env.OFFLINE_GRACE_HOURS ?? 72) || 72));
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
  const signature = await signAccount(payload, signingKey);
  response.status(status).set("cache-control", "no-store").json({
    ok: true,
    message,
    sessionToken,
    account: payload,
    signature,
  });
}

async function recordEvent(userId: string | null, action: string, outcome: string, appVersion: unknown): Promise<void> {
  await db.collection("membership_events").add({
    user_id: userId,
    action,
    outcome,
    app_version: String(appVersion ?? "").trim().slice(0, 32),
    created_at: unixNow(),
  });
}

function requireAdmin(request: Request): void {
  const expected = requiredEnvironment("ADMIN_TOKEN");
  const provided = request.get("authorization")?.replace(/^Bearer\s+/i, "") ?? "";
  if (!constantTimeEqual(provided, expected)) throw new HttpError(401, "unauthorized", "管理凭证无效。");
}

async function getDocument<T>(collectionName: string, documentId: string): Promise<T | null> {
  const result = await db.collection(collectionName).doc(documentId).get();
  return normalizeDocument<T>(result.data);
}

function normalizeDocument<T>(value: T | T[] | null | undefined): T | null {
  if (Array.isArray(value)) return value[0] ?? null;
  return value ?? null;
}

function asyncRoute(handler: (request: Request, response: Response) => Promise<void>) {
  return (request: Request, response: Response, next: NextFunction) => {
    handler(request, response).catch(next);
  };
}

function validateInput<T>(action: () => T): T {
  try {
    return action();
  } catch (error) {
    throw new HttpError(400, "bad_request", error instanceof Error ? error.message : "输入内容不正确。");
  }
}

function requiredEnvironment(name: string): string {
  const value = process.env[name]?.trim() ?? "";
  if (!value) throw new Error(`服务器缺少必要配置：${name}`);
  return value;
}

function isDuplicateDocumentError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return /duplicate|already exists|E11000|重复/i.test(message);
}

function unixNow(): number {
  return Math.floor(Date.now() / 1000);
}

async function hashDeviceId(value: unknown): Promise<string> {
  const normalized = String(value ?? "").trim();
  if (!normalized || normalized.length > 128) throw new HttpError(400, "device_required", "无法识别当前设备，请重新启动软件后重试。");
  return sha256Hex(normalized);
}

function monthKey(now: number): string {
  return new Date(now * 1000).toISOString().slice(0, 7);
}
