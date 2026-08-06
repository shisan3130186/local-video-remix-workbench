use crate::api_config::{protect_data, unprotect_data};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use p256::pkcs8::DecodePublicKey;
use reqwest::{Certificate, Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const STORAGE_VERSION: u8 = 2;
const CONFIG_DIRECTORY: &str = "com.shisan.local-video-remix-workbench";
const STORAGE_FILE_NAME: &str = "secure-membership.bin";
const MAX_STORAGE_BYTES: u64 = 128 * 1024;
const REQUEST_TIMEOUT_SECONDS: u64 = 15;
const API_URL_ENV: &str = "SMARTCUT_MEMBERSHIP_API_URL";
const PUBLIC_KEY_ENV: &str = "SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64";

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
struct StoredAccount {
    version: u8,
    device_id: Option<String>,
    session_token: Option<String>,
    signed_account: Option<SignedAccountPayload>,
    signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SignedAccountPayload {
    schema_version: u8,
    user_id: String,
    email: String,
    display_name: String,
    membership_status: MembershipState,
    expires_at: Option<u64>,
    issued_at: u64,
    offline_until: u64,
    server_time: u64,
    #[serde(default)]
    device_bound: bool,
    #[serde(default = "default_device_match")]
    device_match: bool,
    #[serde(default = "default_rebinds_remaining")]
    rebinds_remaining: u8,
}

fn default_device_match() -> bool {
    true
}
fn default_rebinds_remaining() -> u8 {
    2
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum MembershipState {
    Active,
    Expired,
    None,
    Disabled,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountServerResponse {
    ok: bool,
    message: String,
    code: Option<String>,
    session_token: Option<String>,
    account: Option<SignedAccountPayload>,
    signature: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BasicServerResponse {
    ok: bool,
    message: String,
}

#[derive(Debug)]
struct MembershipRequestError {
    message: String,
    offline_eligible: bool,
    authentication_rejected: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountStatus {
    pub signed_in: bool,
    pub member_active: bool,
    pub membership_state: &'static str,
    pub access_mode: &'static str,
    pub service_configured: bool,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub expires_at: Option<u64>,
    pub offline_until: Option<u64>,
    pub last_validated_at: Option<u64>,
    pub device_bound: bool,
    pub device_match: bool,
    pub rebinds_remaining: u8,
    pub message: String,
}

pub fn get_account_status() -> Result<AccountStatus, String> {
    let stored = normalize_storage(load_storage()?)?;
    Ok(status_from_storage(
        &stored,
        unix_now()?,
        service_configured(),
    ))
}

pub async fn register_account(
    email: String,
    password: String,
    display_name: String,
) -> Result<AccountStatus, String> {
    validate_credentials(&email, &password)?;
    let response = send_public_request(
        "auth/register",
        json!({
            "email": email.trim(),
            "password": password,
            "displayName": display_name.trim(),
            "deviceId": device_id()?,
            "appVersion": env!("CARGO_PKG_VERSION")
        }),
    )
    .await
    .map_err(|error| error.message)?;
    store_authenticated_response(response)
}

pub async fn login_account(email: String, password: String) -> Result<AccountStatus, String> {
    validate_credentials(&email, &password)?;
    let response = send_public_request(
        "auth/login",
        json!({
            "email": email.trim(),
            "password": password,
            "deviceId": device_id()?,
            "appVersion": env!("CARGO_PKG_VERSION")
        }),
    )
    .await
    .map_err(|error| error.message)?;
    store_authenticated_response(response)
}

pub async fn refresh_account() -> Result<AccountStatus, String> {
    let mut stored = normalize_storage(load_storage()?)?;
    let Some(session_token) = stored.session_token.clone() else {
        return Ok(status_from_storage(
            &stored,
            unix_now()?,
            service_configured(),
        ));
    };
    match send_authenticated_request(Method::GET, "account/me", &session_token, None).await {
        Ok(response) => {
            apply_account_response(&mut stored, response, false)?;
            save_storage(&stored)?;
            Ok(mark_online(status_from_storage(&stored, unix_now()?, true)))
        }
        Err(error) if error.offline_eligible => {
            let fallback = status_from_storage(&stored, unix_now()?, service_configured());
            if fallback.signed_in {
                Ok(AccountStatus {
                    message: format!(
                        "暂时无法连接账号服务，正在使用本机签名状态。{}",
                        error.message
                    ),
                    ..fallback
                })
            } else {
                Err(error.message)
            }
        }
        Err(error) => {
            if error.authentication_rejected {
                clear_authentication(&mut stored);
                save_storage(&stored)?;
            }
            Err(error.message)
        }
    }
}

pub async fn redeem_membership(
    redemption_code: String,
    allow_device_rebind: bool,
) -> Result<AccountStatus, String> {
    if redemption_code.trim().chars().count() > 96 {
        return Err("兑换码内容过长，请检查后重新输入。".to_string());
    }
    let mut stored = normalize_storage(load_storage()?)?;
    let session_token = stored
        .session_token
        .clone()
        .ok_or_else(|| "请先登录账号。".to_string())?;
    let response = send_authenticated_request(
        Method::POST,
        "account/redeem",
        &session_token,
        Some(json!({
            "redemptionCode": redemption_code.trim(),
            "allowDeviceRebind": allow_device_rebind,
            "appVersion": env!("CARGO_PKG_VERSION")
        })),
    )
    .await
    .map_err(|error| error.message)?;
    apply_account_response(&mut stored, response, false)?;
    save_storage(&stored)?;
    Ok(mark_online(status_from_storage(&stored, unix_now()?, true)))
}

pub async fn change_account_password(
    current_password: String,
    new_password: String,
) -> Result<AccountStatus, String> {
    validate_password_length(&current_password)?;
    validate_password_length(&new_password)?;
    let mut stored = normalize_storage(load_storage()?)?;
    let session_token = stored
        .session_token
        .clone()
        .ok_or_else(|| "请先登录账号。".to_string())?;
    let response = send_authenticated_request(
        Method::POST,
        "account/change-password",
        &session_token,
        Some(json!({
            "currentPassword": current_password,
            "newPassword": new_password,
            "appVersion": env!("CARGO_PKG_VERSION")
        })),
    )
    .await
    .map_err(|error| error.message)?;
    apply_account_response(&mut stored, response, false)?;
    save_storage(&stored)?;
    Ok(mark_online(status_from_storage(&stored, unix_now()?, true)))
}

pub async fn logout_account() -> Result<AccountStatus, String> {
    let mut stored = normalize_storage(load_storage()?)?;
    if let Some(session_token) = stored.session_token.clone() {
        let _ = send_logout_request(&session_token).await;
    }
    clear_authentication(&mut stored);
    save_storage(&stored)?;
    Ok(status_from_storage(
        &stored,
        unix_now()?,
        service_configured(),
    ))
}

async fn send_public_request(
    action: &str,
    body: Value,
) -> Result<AccountServerResponse, MembershipRequestError> {
    send_request(Method::POST, action, None, Some(body)).await
}

async fn send_authenticated_request(
    method: Method,
    action: &str,
    session_token: &str,
    body: Option<Value>,
) -> Result<AccountServerResponse, MembershipRequestError> {
    send_request(method, action, Some(session_token), body).await
}

async fn send_request(
    method: Method,
    action: &str,
    session_token: Option<&str>,
    body: Option<Value>,
) -> Result<AccountServerResponse, MembershipRequestError> {
    let endpoint = membership_endpoint(action).map_err(configuration_error)?;
    let client = build_client().map_err(configuration_error)?;
    let mut request = client
        .request(method, endpoint)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS));
    let device_id = device_id().map_err(configuration_error)?;
    request = request.header("x-smartcut-device-id", device_id);
    if let Some(token) = session_token {
        request = request.bearer_auth(token);
    }
    if let Some(value) = body {
        request = request.json(&value);
    }
    let response = request.send().await.map_err(transport_error)?;
    parse_account_response(response).await
}

async fn send_logout_request(session_token: &str) -> Result<(), MembershipRequestError> {
    let endpoint = membership_endpoint("auth/logout").map_err(configuration_error)?;
    let response = build_client()
        .map_err(configuration_error)?
        .post(endpoint)
        .bearer_auth(session_token)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .send()
        .await
        .map_err(transport_error)?;
    let status = response.status();
    let parsed = response
        .json::<BasicServerResponse>()
        .await
        .map_err(|_| response_parse_error(status))?;
    if !status.is_success() || !parsed.ok {
        return Err(server_error(status, None, parsed.message));
    }
    Ok(())
}

async fn parse_account_response(
    response: reqwest::Response,
) -> Result<AccountServerResponse, MembershipRequestError> {
    let status = response.status();
    let parsed = response
        .json::<AccountServerResponse>()
        .await
        .map_err(|_| response_parse_error(status))?;
    if !status.is_success() || !parsed.ok {
        return Err(server_error(
            status,
            parsed.code.clone(),
            parsed.message.clone(),
        ));
    }
    Ok(parsed)
}

fn store_authenticated_response(response: AccountServerResponse) -> Result<AccountStatus, String> {
    let mut stored = normalize_storage(load_storage()?)?;
    clear_authentication(&mut stored);
    apply_account_response(&mut stored, response, true)?;
    save_storage(&stored)?;
    Ok(mark_online(status_from_storage(&stored, unix_now()?, true)))
}

fn apply_account_response(
    stored: &mut StoredAccount,
    response: AccountServerResponse,
    session_required: bool,
) -> Result<(), String> {
    let payload = response
        .account
        .ok_or_else(|| "账号服务没有返回签名状态。".to_string())?;
    let signature = response
        .signature
        .ok_or_else(|| "账号服务没有返回状态签名。".to_string())?;
    verify_signed_account(&payload, &signature, &public_key_base64()?)?;
    if let Some(session_token) = response.session_token {
        if session_token.len() < 32 || session_token.len() > 256 {
            return Err("账号服务返回的登录凭证无效。".to_string());
        }
        stored.session_token = Some(session_token);
    } else if session_required {
        return Err("账号服务没有返回登录凭证。".to_string());
    }
    stored.signed_account = Some(payload);
    stored.signature = Some(signature);
    Ok(())
}

fn verify_signed_account(
    payload: &SignedAccountPayload,
    signature_base64: &str,
    public_key_base64: &str,
) -> Result<(), String> {
    if payload.schema_version != 1 {
        return Err("账号状态版本不受支持，请升级软件。".to_string());
    }
    if payload.email.trim().is_empty() || payload.user_id.trim().is_empty() {
        return Err("账号签名内容不完整。".to_string());
    }
    if payload.offline_until < payload.issued_at {
        return Err("账号签名时间范围异常。".to_string());
    }
    if let Some(expires_at) = payload.expires_at {
        if payload.offline_until > expires_at
            && payload.membership_status == MembershipState::Active
        {
            return Err("会员离线时间超过到期时间。".to_string());
        }
    }
    let public_der = BASE64
        .decode(public_key_base64.trim())
        .map_err(|_| "会员验证公钥配置无效。".to_string())?;
    let verifying_key = VerifyingKey::from_public_key_der(&public_der)
        .map_err(|_| "会员验证公钥无法读取。".to_string())?;
    let signature_bytes = BASE64
        .decode(signature_base64.trim())
        .map_err(|_| "账号状态签名格式无效。".to_string())?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| "账号状态签名长度无效。".to_string())?;
    let message =
        serde_json::to_vec(payload).map_err(|error| format!("无法整理账号状态：{error}"))?;
    verifying_key
        .verify(&message, &signature)
        .map_err(|_| "账号状态签名校验失败，本机缓存可能已被修改。".to_string())
}

fn status_from_storage(stored: &StoredAccount, now: u64, configured: bool) -> AccountStatus {
    let Some(payload) = stored.signed_account.as_ref() else {
        return signed_out_status(configured);
    };
    if stored.session_token.is_none() {
        return signed_out_status(configured);
    }
    let verified = stored
        .signature
        .as_deref()
        .zip(public_key_base64().ok())
        .and_then(|(signature, key)| verify_signed_account(payload, signature, &key).ok())
        .is_some();
    if !verified {
        return AccountStatus {
            signed_in: false,
            member_active: false,
            membership_state: "invalid",
            access_mode: "blocked",
            service_configured: configured,
            email: None,
            display_name: None,
            expires_at: payload.expires_at,
            offline_until: Some(payload.offline_until),
            last_validated_at: Some(payload.issued_at),
            device_bound: payload.device_bound,
            device_match: payload.device_match,
            rebinds_remaining: payload.rebinds_remaining,
            message: "本机账号缓存校验失败，请重新登录。".to_string(),
        };
    }

    let (member_active, membership_state, access_mode, message) = match payload.membership_status {
        MembershipState::Active if payload.expires_at.is_some_and(|expires| now > expires) => {
            (false, "expired", "cached", "会员已到期。")
        }
        MembershipState::Active if payload.device_bound && !payload.device_match => (
            false,
            "verification_required",
            "blocked",
            "当前会员已绑定其他设备，请兑换后确认换绑当前设备。",
        ),
        MembershipState::Active if now <= payload.offline_until => (
            true,
            "active",
            "offline",
            "会员可用，当前使用本机签名状态。",
        ),
        MembershipState::Active => (
            false,
            "verification_required",
            "blocked",
            "离线宽限已结束，请联网刷新账号状态。",
        ),
        MembershipState::Expired => (false, "expired", "cached", "会员已到期。"),
        MembershipState::None => (
            false,
            "none",
            "cached",
            "当前是普通用户，可以使用兑换码开通会员。",
        ),
        MembershipState::Disabled => (false, "disabled", "blocked", "账号或会员已被停用。"),
    };
    AccountStatus {
        signed_in: true,
        member_active,
        membership_state,
        access_mode,
        service_configured: configured,
        email: Some(payload.email.clone()),
        display_name: Some(payload.display_name.clone()),
        expires_at: payload.expires_at,
        offline_until: Some(payload.offline_until),
        last_validated_at: Some(payload.issued_at),
        device_bound: payload.device_bound,
        device_match: payload.device_match,
        rebinds_remaining: payload.rebinds_remaining,
        message: message.to_string(),
    }
}

fn signed_out_status(configured: bool) -> AccountStatus {
    AccountStatus {
        signed_in: false,
        member_active: false,
        membership_state: "signed_out",
        access_mode: "signed_out",
        service_configured: configured,
        email: None,
        display_name: None,
        expires_at: None,
        offline_until: None,
        last_validated_at: None,
        device_bound: false,
        device_match: true,
        rebinds_remaining: 2,
        message: if configured {
            "登录账号后可以兑换会员并在其他电脑恢复状态。".to_string()
        } else {
            "账号服务尚未完成部署配置。".to_string()
        },
    }
}

fn mark_online(mut status: AccountStatus) -> AccountStatus {
    if status.signed_in {
        status.access_mode = "online";
        status.message = if status.member_active {
            "会员状态正常，已完成联网验证。".to_string()
        } else {
            "账号状态正常。".to_string()
        };
    }
    status
}

fn clear_authentication(stored: &mut StoredAccount) {
    stored.session_token = None;
    stored.signed_account = None;
    stored.signature = None;
}

fn normalize_storage(mut stored: StoredAccount) -> Result<StoredAccount, String> {
    if stored.version == 0 {
        stored.version = STORAGE_VERSION;
    }
    if stored.version != STORAGE_VERSION {
        stored = StoredAccount {
            version: STORAGE_VERSION,
            ..StoredAccount::default()
        };
        save_storage(&stored)?;
    }
    if stored.device_id.as_deref().is_none_or(str::is_empty) {
        stored.device_id = Some(Uuid::new_v4().to_string());
        save_storage(&stored)?;
    }
    Ok(stored)
}

fn device_id() -> Result<String, String> {
    let stored = normalize_storage(load_storage()?)?;
    stored
        .device_id
        .ok_or_else(|| "Unable to generate a local device identifier.".to_string())
}

fn load_storage() -> Result<StoredAccount, String> {
    load_storage_from_path(&storage_file_path()?)
}

fn load_storage_from_path(path: &Path) -> Result<StoredAccount, String> {
    if !path.exists() {
        return Ok(StoredAccount::default());
    }
    let metadata = fs::metadata(path).map_err(|error| format!("无法读取本机账号配置：{error}"))?;
    if metadata.len() > MAX_STORAGE_BYTES {
        return Err("本机账号配置异常，请删除后重新登录。".to_string());
    }
    let encrypted = fs::read(path).map_err(|error| format!("无法读取本机账号配置：{error}"))?;
    let mut plaintext = unprotect_data(&encrypted)?;
    let parsed = serde_json::from_slice::<StoredAccount>(&plaintext)
        .map_err(|error| format!("本机账号配置无法识别：{error}"));
    plaintext.fill(0);
    parsed
}

fn save_storage(stored: &StoredAccount) -> Result<(), String> {
    let path = storage_file_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定账号配置目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建账号配置目录：{error}"))?;
    let mut plaintext =
        serde_json::to_vec(stored).map_err(|error| format!("无法整理账号配置：{error}"))?;
    let encrypted_result = protect_data(&plaintext);
    plaintext.fill(0);
    fs::write(path, encrypted_result?).map_err(|error| format!("无法保存账号配置：{error}"))
}

fn storage_file_path() -> Result<PathBuf, String> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法找到Windows应用配置目录。".to_string())?;
    Ok(app_data.join(CONFIG_DIRECTORY).join(STORAGE_FILE_NAME))
}

fn membership_endpoint(action: &str) -> Result<String, String> {
    let base_url = membership_api_url().ok_or_else(|| "账号服务尚未部署。".to_string())?;
    Ok(format!("{}/v1/{action}", base_url.trim_end_matches('/')))
}

fn membership_api_url() -> Option<String> {
    development_environment_value(API_URL_ENV).or_else(|| {
        option_env!("SMARTCUT_MEMBERSHIP_API_URL")
            .map(str::to_string)
            .filter(|value| !value.trim().is_empty())
    })
}

fn public_key_base64() -> Result<String, String> {
    development_environment_value(PUBLIC_KEY_ENV)
        .or_else(|| {
            option_env!("SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64")
                .map(str::to_string)
                .filter(|value| !value.trim().is_empty())
        })
        .ok_or_else(|| "会员验证公钥尚未配置。".to_string())
}

#[cfg(debug_assertions)]
fn development_environment_value(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

#[cfg(not(debug_assertions))]
fn development_environment_value(_name: &str) -> Option<String> {
    None
}

fn service_configured() -> bool {
    membership_api_url().is_some() && public_key_base64().is_ok()
}

fn build_client() -> Result<Client, String> {
    let roots = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|certificate| Certificate::from_der(certificate.as_ref()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("无法加载内置HTTPS根证书：{error}"))?;
    Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .http1_only()
        .tls_certs_only(roots)
        .build()
        .map_err(|error| format!("无法创建账号服务连接：{error}"))
}

fn transport_error(error: reqwest::Error) -> MembershipRequestError {
    MembershipRequestError {
        message: if error.is_timeout() {
            "连接账号服务超时，请检查网络后重试。".to_string()
        } else if error.is_connect() {
            "暂时无法连接账号服务，请检查网络后重试。".to_string()
        } else {
            format!("账号服务请求失败：{error}")
        },
        offline_eligible: true,
        authentication_rejected: false,
    }
}

fn response_parse_error(status: StatusCode) -> MembershipRequestError {
    MembershipRequestError {
        message: "账号服务返回了无法识别的结果。".to_string(),
        offline_eligible: status.is_server_error(),
        authentication_rejected: false,
    }
}

fn server_error(
    status: StatusCode,
    code: Option<String>,
    message: String,
) -> MembershipRequestError {
    MembershipRequestError {
        message: if message.trim().is_empty() {
            if status.is_server_error() {
                "账号服务暂时不可用，请稍后重试。".to_string()
            } else {
                "账号操作失败。".to_string()
            }
        } else {
            message
        },
        offline_eligible: status.is_server_error(),
        authentication_rejected: status == StatusCode::UNAUTHORIZED
            || matches!(
                code.as_deref(),
                Some("session_invalid" | "session_required")
            ),
    }
}

fn configuration_error(message: String) -> MembershipRequestError {
    MembershipRequestError {
        message,
        offline_eligible: false,
        authentication_rejected: false,
    }
}

fn validate_credentials(email: &str, password: &str) -> Result<(), String> {
    if email.trim().chars().count() < 5 || email.chars().count() > 254 || !email.contains('@') {
        return Err("请输入正确的邮箱地址。".to_string());
    }
    validate_password_length(password)
}

fn validate_password_length(password: &str) -> Result<(), String> {
    let length = password.chars().count();
    if !(8..=128).contains(&length) {
        return Err("密码必须是8到128个字符。".to_string());
    }
    Ok(())
}

fn unix_now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "系统时间异常，无法校验会员有效期。".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        status_from_storage, verify_signed_account, MembershipState, SignedAccountPayload,
        StoredAccount,
    };
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use p256::ecdsa::{signature::Signer, Signature, SigningKey};
    use p256::pkcs8::EncodePublicKey;

    fn signed_fixture(status: MembershipState) -> (SignedAccountPayload, String, String) {
        let signing_key = SigningKey::from_bytes((&[9_u8; 32]).into()).unwrap();
        let payload = SignedAccountPayload {
            schema_version: 1,
            user_id: "user-1".to_string(),
            email: "user@example.com".to_string(),
            display_name: "智剪用户".to_string(),
            membership_status: status,
            expires_at: Some(10_000),
            issued_at: 1_000,
            offline_until: 2_000,
            server_time: 1_000,
            device_bound: false,
            device_match: true,
            rebinds_remaining: 2,
        };
        let signature: Signature = signing_key.sign(&serde_json::to_vec(&payload).unwrap());
        let public_der = signing_key.verifying_key().to_public_key_der().unwrap();
        (
            payload,
            BASE64.encode(signature.to_bytes()),
            BASE64.encode(public_der.as_bytes()),
        )
    }

    #[test]
    fn signed_account_rejects_tampering() {
        let (mut payload, signature, public_key) = signed_fixture(MembershipState::Active);
        verify_signed_account(&payload, &signature, &public_key).unwrap();
        payload.display_name = "被修改".to_string();
        assert!(verify_signed_account(&payload, &signature, &public_key).is_err());
    }

    #[test]
    fn account_membership_only_works_inside_signed_offline_window() {
        let (payload, signature, public_key) = signed_fixture(MembershipState::Active);
        std::env::set_var("SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64", public_key);
        let stored = StoredAccount {
            version: 2,
            device_id: Some("test-device".to_string()),
            session_token: Some("session-token-with-enough-characters-123456".to_string()),
            signed_account: Some(payload),
            signature: Some(signature),
        };
        assert!(status_from_storage(&stored, 1_500, true).member_active);
        assert!(!status_from_storage(&stored, 2_001, true).member_active);
        std::env::remove_var("SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64");
    }

    #[test]
    fn ordinary_account_stays_signed_in_without_member_access() {
        let (mut payload, _signature, public_key) = signed_fixture(MembershipState::None);
        payload.expires_at = None;
        payload.offline_until = payload.issued_at;
        let signing_key = SigningKey::from_bytes((&[9_u8; 32]).into()).unwrap();
        let signature: Signature = signing_key.sign(&serde_json::to_vec(&payload).unwrap());
        std::env::set_var("SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64", public_key);
        let stored = StoredAccount {
            version: 2,
            device_id: Some("test-device".to_string()),
            session_token: Some("session-token-with-enough-characters-123456".to_string()),
            signed_account: Some(payload),
            signature: Some(BASE64.encode(signature.to_bytes())),
        };
        let status = status_from_storage(&stored, 1_500, true);
        assert!(status.signed_in);
        assert!(!status.member_active);
        std::env::remove_var("SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64");
    }
}
