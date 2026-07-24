import assert from "node:assert/strict";
import test from "node:test";
import {
  buildSignedAccount,
  constantTimeEqual,
  formatRedemptionCode,
  hashPassword,
  normalizeRedemptionCode,
  resolveMembershipState,
  safeDurationDays,
  validateEmail,
  validatePassword,
} from "../src/shared.ts";

test("邮箱统一转小写并拒绝错误格式", () => {
  assert.equal(validateEmail(" User@Example.com "), "user@example.com");
  assert.throws(() => validateEmail("not-an-email"));
});

test("密码必须至少8个字符", () => {
  assert.equal(validatePassword("12345678"), "12345678");
  assert.throws(() => validatePassword("1234567"));
});

test("同一密码和盐生成相同哈希，错误密码不同", async () => {
  const first = await hashPassword("correct-password");
  const same = await hashPassword("correct-password", first.salt, first.iterations);
  const wrong = await hashPassword("wrong-password", first.salt, first.iterations);
  assert.equal(constantTimeEqual(first.hash, same.hash), true);
  assert.equal(constantTimeEqual(first.hash, wrong.hash), false);
});

test("兑换码格式包含智剪前缀并避开混淆字符", () => {
  const code = formatRedemptionCode(Uint8Array.from({ length: 16 }, (_, index) => index));
  assert.match(code, /^SCUT-[A-HJ-NP-Z2-9]{4}(?:-[A-HJ-NP-Z2-9]{4}){3}$/);
  assert.equal(normalizeRedemptionCode(code).length, 20);
});

test("账号或会员停用优先阻止会员状态", () => {
  assert.equal(resolveMembershipState("disabled", "active", 999, 100), "disabled");
  assert.equal(resolveMembershipState("active", "disabled", 999, 100), "disabled");
  assert.equal(resolveMembershipState("active", null, null, 100), "none");
  assert.equal(resolveMembershipState("active", "active", 99, 100), "expired");
});

test("签名账号的离线时间不能超过会员到期时间", () => {
  const account = buildSignedAccount({
    userId: "user-1",
    email: "user@example.com",
    displayName: "智剪用户",
    membershipStatus: "active",
    expiresAt: 200,
    now: 100,
    graceHours: 72,
  });
  assert.equal(account.offlineUntil, 200);
});

test("普通用户没有离线会员权限", () => {
  const account = buildSignedAccount({
    userId: "user-1",
    email: "user@example.com",
    displayName: "智剪用户",
    membershipStatus: "none",
    expiresAt: null,
    now: 100,
    graceHours: 72,
  });
  assert.equal(account.offlineUntil, 100);
});

test("兑换天数限制在安全范围", () => {
  assert.equal(safeDurationDays(30), 30);
  assert.throws(() => safeDurationDays(0));
  assert.throws(() => safeDurationDays(3651));
});
