use anyhow::{anyhow, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tauri::WebviewWindow;
#[cfg(not(target_os = "android"))]
use uuid::Uuid;

use crate::domain::{DetectedPhone, OfficialAccount, QuotaInfo, RegisterOfficialAccountRequest};
#[cfg(not(target_os = "android"))]
use crate::storage::{load_global_setting, save_global_setting};
use crate::storage::{
    load_official_account, mask_phone, official_api_base_url, save_official_account, AppState,
};

#[cfg(not(target_os = "android"))]
const DEV_ANDROID_ID_KEY: &str = "dev_android_id";
const PHONE_PERMISSION_REQUESTED: &str = "__PERMISSION_REQUESTED__";

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    phone: String,
    user_id: String,
    pool_balance: i64,
}

#[derive(Debug, Deserialize)]
struct InviteCodeResponse {
    phone: String,
    invite_code: String,
}

pub fn account(state: &AppState, window: &WebviewWindow) -> Result<OfficialAccount> {
    let android_id = android_id(state, window)?;
    load_official_account(state, android_id)
}

pub fn detect_phone(state: &AppState, window: &WebviewWindow) -> Result<DetectedPhone> {
    let _ = android_id(state, window)?;
    let phone = primary_phone(window)?;
    Ok(DetectedPhone {
        masked_phone: mask_phone(&phone),
    })
}

pub async fn register(
    state: &AppState,
    window: &WebviewWindow,
    request: RegisterOfficialAccountRequest,
) -> Result<OfficialAccount> {
    let middle_four = request.middle_four.trim();
    if middle_four.len() != 4 || !middle_four.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(anyhow!("请输入手机号中间 4 位数字。"));
    }

    let android_id = android_id(state, window)?;
    let phone = primary_phone(window)?;
    let expected = phone
        .get(3..7)
        .ok_or_else(|| anyhow!("系统读取到的手机号格式异常，无法注册。"))?;
    if middle_four != expected {
        return Err(anyhow!("手机号中间 4 位不匹配，无法注册。"));
    }

    let invite_code = request
        .invite_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let client = Client::new();
    let response = client
        .post(format!("{}/register", official_api_base_url()))
        .json(&serde_json::json!({
            "phone": &phone,
            "invite_code": invite_code
        }))
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if status == StatusCode::CONFLICT && text.contains("phone already registered") {
        let invite_code = fetch_invite_code(&client, &android_id, &phone).await?;
        let account = OfficialAccount {
            android_id,
            masked_phone: Some(mask_phone(&phone)),
            phone: Some(phone),
            invite_code: Some(invite_code),
            user_id: None,
            pool_balance: None,
            registered: true,
        };
        save_official_account(state, &account)?;
        return Ok(account);
    }
    if !status.is_success() {
        return Err(anyhow!("官方注册失败 ({status}): {text}"));
    }
    let registered: RegisterResponse = serde_json::from_str(&text)
        .map_err(|err| anyhow!("官方注册响应无法解析: {err}; body={text}"))?;
    let invite_code = fetch_invite_code(&client, &android_id, &registered.phone).await?;

    let account = OfficialAccount {
        android_id,
        masked_phone: Some(mask_phone(&registered.phone)),
        phone: Some(registered.phone),
        invite_code: Some(invite_code),
        user_id: Some(registered.user_id),
        pool_balance: Some(registered.pool_balance),
        registered: true,
    };
    save_official_account(state, &account)?;
    Ok(account)
}

async fn fetch_invite_code(client: &Client, android_id: &str, phone: &str) -> Result<String> {
    let response = client
        .get(format!("{}/invite-code", official_api_base_url()))
        .bearer_auth(format!("{android_id}-{phone}"))
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(anyhow!("邀请码查询失败 ({status}): {text}"));
    }
    let invite: InviteCodeResponse = serde_json::from_str(&text)
        .map_err(|err| anyhow!("邀请码响应无法解析: {err}; body={text}"))?;
    if invite.phone != phone {
        return Err(anyhow!("邀请码响应手机号不匹配，无法登录官方账号。"));
    }
    Ok(invite.invite_code)
}

pub async fn quota(state: &AppState, window: &WebviewWindow) -> Result<QuotaInfo> {
    let android_id = android_id(state, window)?;
    let token = crate::storage::official_auth_token(state, android_id)?;
    let client = Client::new();
    let response = client
        .get(format!("{}/quota", official_api_base_url()))
        .bearer_auth(token)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(anyhow!("额度查询失败 ({status}): {text}"));
    }
    serde_json::from_str(&text).map_err(|err| anyhow!("额度响应无法解析: {err}; body={text}"))
}

pub fn android_id(state: &AppState, window: &WebviewWindow) -> Result<String> {
    let id = platform_android_id(state, window)?;
    if id.trim().is_empty() {
        Err(anyhow!("无法读取 Android ID。"))
    } else {
        Ok(id.trim().to_string())
    }
}

fn normalize_phone(raw: String) -> Result<String> {
    if raw == PHONE_PERMISSION_REQUESTED {
        return Err(anyhow!("已申请手机号验证权限，请授权后再次点击验证手机号。"));
    }
    let digits: String = raw.chars().filter(|ch| ch.is_ascii_digit()).collect();
    let normalized = if digits.len() > 11 {
        digits[digits.len() - 11..].to_string()
    } else {
        digits
    };
    if normalized.len() != 11 {
        return Err(anyhow!("无法稳定验证本机手机号，不能注册或登录官方账号。"));
    }
    Ok(normalized)
}

fn primary_phone(window: &WebviewWindow) -> Result<String> {
    normalize_phone(platform_primary_phone(window)?)
}

#[cfg(target_os = "android")]
fn platform_android_id(_state: &AppState, window: &WebviewWindow) -> Result<String> {
    call_android_string_method(window, "getAndroidId")
}

#[cfg(not(target_os = "android"))]
fn platform_android_id(state: &AppState, _window: &WebviewWindow) -> Result<String> {
    if let Some(id) = load_global_setting(state, DEV_ANDROID_ID_KEY)? {
        return Ok(id);
    }
    let id = format!("dev_{}", Uuid::new_v4().simple());
    save_global_setting(state, DEV_ANDROID_ID_KEY, &id)?;
    Ok(id)
}

#[cfg(target_os = "android")]
fn platform_primary_phone(window: &WebviewWindow) -> Result<String> {
    call_android_string_method(window, "readPrimaryPhoneNumber")
}

#[cfg(not(target_os = "android"))]
fn platform_primary_phone(_window: &WebviewWindow) -> Result<String> {
    Err(anyhow!("手机号注册只支持 Android 真机。"))
}

#[cfg(target_os = "android")]
fn call_android_string_method(window: &WebviewWindow, method: &str) -> Result<String> {
    use jni::objects::{JObject, JString};

    let method = method.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    window.with_webview(move |platform| {
        platform.jni_handle().exec(move |env, activity, _webview| {
            let result = (|| {
                let value = env
                    .call_method(activity, &method, "()Ljava/lang/String;", &[])?
                    .l()?;
                if value.is_null() {
                    return Ok(String::new());
                }
                let value = JString::from(JObject::from(value));
                let value: String = env.get_string(&value)?.into();
                Ok(value)
            })();
            let _ = tx.send(result);
        });
    })?;
    rx.recv()
        .map_err(|_| anyhow!("Android identity reader did not respond"))?
}
