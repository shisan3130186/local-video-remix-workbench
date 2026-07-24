# 智剪账号会员服务

正式第一版使用腾讯云开发 CloudBase HTTP 云函数和文档数据库；Cloudflare Workers + D1 仅保留为开发备份。账号流程包括邮箱注册/登录、兑换码充值、个人中心显示有效期、修改密码和退出登录。

## 安全边界

- 密码使用 PBKDF2-SHA256 独立随机盐保存，服务端和桌面端均不保存明文密码。
- 登录会话只保存令牌哈希；完整会话令牌只由 Windows 当前用户加密保存在本机。
- 兑换码明文只在创建时显示一次，数据库只保存 SHA-256 哈希。
- 会员状态由 ECDSA P-256 私钥签名，桌面软件只内置公钥。
- 在线验证后最多离线使用 72 小时，且不能超过会员到期时间。

## 接口

- `POST /v1/auth/register`
- `POST /v1/auth/login`
- `POST /v1/auth/logout`
- `GET /v1/account/me`
- `POST /v1/account/redeem`
- `POST /v1/account/change-password`
- `GET/POST/PATCH /v1/admin/codes`

## CloudBase首次部署

1. 登录腾讯云开发并创建环境，把环境ID写入`cloudbaserc.json`。
2. 创建`users`、`sessions`、`memberships`、`redemption_codes`和`membership_events`集合。
3. 运行`scripts/configure-cloudbase-secrets.ps1`，私钥和管理令牌只以Windows加密形式保存在本机。
4. 运行`pnpm run deploy:cloudbase`部署HTTP云函数。
5. 在国内网络完成注册、登录、兑换、改密和会员恢复回归，再把公开验签公钥编译进桌面端。

## 管理兑换码

`scripts/admin-license.ps1` 支持查看、创建、停用和恢复未使用的兑换码。管理令牌可以通过 `-TokenFile` 从当前 Windows 账户加密文件读取。

零基础操作优先双击项目根目录的`启动智剪卡密管理器.vbs`。可视化管理器支持预设/自定义会员天数、备注、生成并复制、状态筛选、停用和恢复，且不会显示管理员令牌。
