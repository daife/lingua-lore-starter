use serde::Serialize;
use tauri::AppHandle;

const STARTER_RELEASE_URL: &str =
    "https://api.github.com/repos/daife/lingua-lore-starter/releases/latest";
const ANNOUNCEMENT_URL: &str =
    "https://raw.githubusercontent.com/daife/lingua-lore-starter/main/announcements/android.md";

#[derive(Debug, Serialize)]
pub struct CheckVersionResult {
    pub has_update: bool,
    pub latest_version: String,
}

#[derive(Debug, Serialize)]
pub struct AnnouncementResult {
    pub content: String,
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
}

/// Compare two version strings (e.g. "0.1.8" vs "0.1.9").
/// Returns true when `latest` is strictly greater than `current`.
fn is_newer_version(current: &str, latest: &str) -> bool {
    let latest = latest.strip_prefix('v').unwrap_or(latest);
    let cur_parts: Vec<u32> = current.split('.').filter_map(|p| p.parse().ok()).collect();
    let lat_parts: Vec<u32> = latest.split('.').filter_map(|p| p.parse().ok()).collect();

    let max_len = cur_parts.len().max(lat_parts.len());
    for i in 0..max_len {
        let c = cur_parts.get(i).copied().unwrap_or(0);
        let l = lat_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }
    false
}

/// Check GitHub for a newer release. On any error (network, parse, etc.)
/// returns `{ has_update: false }` so the app starts normally.
#[tauri::command]
pub async fn check_version() -> CheckVersionResult {
    let current_version = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent("LinguaLore/1.0")
        .timeout(std::time::Duration::from_secs(8))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => {
            return CheckVersionResult {
                has_update: false,
                latest_version: String::new(),
            }
        }
    };

    let resp = client.get(STARTER_RELEASE_URL).send().await;

    let resp = match resp {
        Ok(r) => r,
        Err(_) => {
            return CheckVersionResult {
                has_update: false,
                latest_version: String::new(),
            }
        }
    };

    let release: GithubRelease = match resp.json().await {
        Ok(r) => r,
        Err(_) => {
            return CheckVersionResult {
                has_update: false,
                latest_version: String::new(),
            }
        }
    };

    let has_update = is_newer_version(current_version, &release.tag_name);

    CheckVersionResult {
        has_update,
        latest_version: release.tag_name,
    }
}

#[tauri::command]
pub async fn check_announcement() -> AnnouncementResult {
    let client = reqwest::Client::builder()
        .user_agent("LinguaLore/1.0")
        .timeout(std::time::Duration::from_secs(8))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => {
            return AnnouncementResult {
                content: String::new(),
            }
        }
    };

    let resp = client.get(ANNOUNCEMENT_URL).send().await;
    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        _ => {
            return AnnouncementResult {
                content: String::new(),
            }
        }
    };

    let content = resp.text().await.unwrap_or_default();
    AnnouncementResult {
        content: content.trim().to_string(),
    }
}

/// Force-quit the application.
#[tauri::command]
pub fn quit_app(app_handle: AppHandle) {
    app_handle.exit(0);
}
