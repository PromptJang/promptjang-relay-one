use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const RELEASES_URL: &str =
    "https://api.github.com/repos/PromptJang/promptjang-relay-one/releases/latest";
const RELEASE_PAGE: &str = "https://github.com/PromptJang/promptjang-relay-one/releases/latest";
const RELEASE_URL_PREFIX: &str = "https://github.com/PromptJang/promptjang-relay-one/releases/";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateInfo {
    pub enabled: bool,
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_url: String,
    pub release_notes: Option<String>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
    pub check_error: Option<String>,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[derive(Clone)]
struct Cached {
    at: Instant,
    path: PathBuf,
    info: UpdateInfo,
}
static CACHE: OnceLock<Mutex<Option<Cached>>> = OnceLock::new();

pub async fn check(enabled: bool, refresh: bool, cache_path: &Path) -> UpdateInfo {
    if !enabled {
        return disabled();
    }
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().await;
    if !refresh
        && let Some(cached) = guard.as_ref().filter(|entry| {
            entry.path == cache_path && entry.at.elapsed() < Duration::from_secs(86_400)
        })
    {
        return cached.info.clone();
    }
    if !refresh && let Some(info) = read_cache(cache_path) {
        *guard = Some(Cached {
            at: Instant::now(),
            path: cache_path.to_path_buf(),
            info: info.clone(),
        });
        return info;
    }
    let current = env!("CARGO_PKG_VERSION").to_string();
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(error) => return failure(current, error.to_string()),
    };
    let result = client
        .get(RELEASES_URL)
        .header("User-Agent", "promptjang-relay-one")
        .send()
        .await
        .and_then(reqwest::Response::error_for_status);
    let info = match result {
        Ok(response) => match response.json::<GitHubRelease>().await {
            Ok(release) => {
                if !release.html_url.starts_with(RELEASE_URL_PREFIX) {
                    return failure(
                        current,
                        "release URL was outside the official repository".into(),
                    );
                }
                let latest = release.tag_name.trim_start_matches('v').to_string();
                let available = semver::Version::parse(&latest)
                    .ok()
                    .zip(semver::Version::parse(&current).ok())
                    .is_some_and(|(latest, current)| latest > current);
                UpdateInfo {
                    enabled: true,
                    available,
                    current_version: current,
                    latest_version: Some(latest),
                    release_url: release.html_url,
                    release_notes: release.body,
                    checked_at: chrono::Utc::now(),
                    check_error: None,
                }
            }
            Err(error) => failure(current, error.to_string()),
        },
        Err(error) => failure(current, error.to_string()),
    };
    *guard = Some(Cached {
        at: Instant::now(),
        path: cache_path.to_path_buf(),
        info: info.clone(),
    });
    write_cache(cache_path, &info);
    info
}

fn read_cache(path: &Path) -> Option<UpdateInfo> {
    let info = serde_json::from_slice::<UpdateInfo>(&std::fs::read(path).ok()?).ok()?;
    let age = chrono::Utc::now().signed_duration_since(info.checked_at);
    (age >= chrono::Duration::zero() && age < chrono::Duration::hours(24)).then_some(info)
}

fn write_cache(path: &Path, info: &UpdateInfo) {
    let result = serde_json::to_vec(info)
        .map_err(anyhow::Error::from)
        .and_then(|json| std::fs::write(path, json).map_err(anyhow::Error::from));
    if let Err(error) = result {
        tracing::warn!(%error, "could not persist update cache");
    }
}

fn failure(current_version: String, message: String) -> UpdateInfo {
    tracing::warn!(error=%message, "update check failed");
    UpdateInfo {
        enabled: true,
        available: false,
        current_version,
        latest_version: None,
        release_url: RELEASE_PAGE.into(),
        release_notes: None,
        checked_at: chrono::Utc::now(),
        check_error: Some("Update check unavailable".into()),
    }
}

fn disabled() -> UpdateInfo {
    UpdateInfo {
        enabled: false,
        available: false,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        latest_version: None,
        release_url: RELEASE_PAGE.into(),
        release_notes: None,
        checked_at: chrono::Utc::now(),
        check_error: None,
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn disabled_update_checks_do_not_need_a_release_request() {
        let cache =
            std::env::temp_dir().join(format!("pj-one-update-{}.json", uuid::Uuid::new_v4()));
        let result = super::check(false, false, &cache).await;
        assert!(!result.enabled);
        assert!(!result.available);
        assert!(result.check_error.is_none());
    }

    #[test]
    fn recent_cache_round_trips_without_network_state() {
        let cache =
            std::env::temp_dir().join(format!("pj-one-update-{}.json", uuid::Uuid::new_v4()));
        let info = super::disabled();
        super::write_cache(&cache, &info);
        let restored = super::read_cache(&cache).expect("recent cache");
        assert_eq!(restored.current_version, info.current_version);
        std::fs::remove_file(cache).expect("remove update cache");
    }

    #[test]
    fn release_endpoint_is_pinned_to_promptjang() {
        assert_eq!(
            super::RELEASES_URL,
            "https://api.github.com/repos/PromptJang/promptjang-relay-one/releases/latest"
        );
    }
}
