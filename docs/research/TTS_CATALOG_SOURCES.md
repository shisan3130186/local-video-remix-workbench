# 豆包音色目录来源

更新时间：2026-08-06

## 使用规则

- 目录只保存公开的 speaker ID、展示名称和筛选信息，不保存用户 API Key、账户音色或请求结果。
- 官方样音是加速试听的可选资源，不能作为可用性判断；失败时通过用户已经配置的豆包 TTS 实时合成。
- 头像优先使用已有本地资源或官方 CDN，失败时由前端生成渐变文字头像，避免把大量头像文件打入安装包。

## 公开资料

1. 火山引擎 V3 HTTP TTS：<https://www.volcengine.com/docs/6561/1598757>
   - 确认 `X-Api-Resource-Id` 与 speaker 音色家族必须匹配。
2. 火山引擎音色管理接口：<https://www.volcengine.com/docs/6561/2160690>
   - 用于后续按用户账号权限拉取可用音色，本轮不发送用户凭证。
3. 火山公开 2.0 音色交叉清单：<https://github.com/LonePheasantWarrior/TalkifyTTS/blob/a66f69e46b718ab2cd6d72c14d37166c75267682/doc/voices/%E8%B1%86%E5%8C%85%E8%AF%AD%E9%9F%B3%E5%90%88%E6%88%90%E6%A8%A1%E5%9E%8B2.0%E9%9F%B3%E8%89%B2%E5%88%97%E8%A1%A8.md>
4. 火山旧版公开音色交叉清单：<https://github.com/labring/sealos/blob/af8a321511bd/service/aiproxy/relay/adaptor/doubaoaudio/constants.go>

## 兼容决策

- `*_uranus_bigtts`、`saturn_*`：默认使用 `seed-tts-2.0`。
- `*_mars_bigtts`、`*_moon_bigtts`、非 Uranus 的 `ICL_*`：使用 `seed-tts-1.0`。
- `ICL_uranus_*_tob` 属于 2.0 角色音色，使用 `seed-tts-2.0`；已知音色家族会覆盖错误的旧配置，避免资源 ID 与 speaker 错配。
- 未知 speaker 才保留用户配置的资源 ID；真实可用范围仍由其火山账号权限、资源包和余额决定。
