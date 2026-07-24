import { webcrypto } from "node:crypto";

const pair = await webcrypto.subtle.generateKey(
  { name: "ECDSA", namedCurve: "P-256" },
  true,
  ["sign", "verify"],
);
const privateKey = Buffer.from(await webcrypto.subtle.exportKey("pkcs8", pair.privateKey)).toString("base64");
const publicKey = Buffer.from(await webcrypto.subtle.exportKey("spki", pair.publicKey)).toString("base64");

if (process.argv.includes("--json")) {
  process.stdout.write(JSON.stringify({ privateKey, publicKey }));
  process.exit(0);
}

process.stdout.write([
  "请把下面私钥设置为会员服务端环境变量：SIGNING_PRIVATE_KEY_PKCS8_BASE64",
  privateKey,
  "",
  "请把下面公钥写入桌面端构建环境：SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64",
  publicKey,
  "",
].join("\n"));
