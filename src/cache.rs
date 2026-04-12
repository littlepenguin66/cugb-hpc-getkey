use crate::paths::cache_file_path;
use crate::types::TokenCache;
use std::fs;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    Missing,
    Invalid,
    Expired,
    Valid,
}

pub fn read_cache(username: &str) -> Option<String> {
    let cache_file = cache_file_path();

    let text = fs::read_to_string(&cache_file).ok()?;
    let cache: TokenCache = serde_json::from_str(&text).ok()?;

    if cache.username != username {
        return None;
    }

    let now = chrono::Local::now().timestamp_millis();
    if now >= cache.expires_at {
        return None;
    }

    Some(cache.token)
}

pub fn write_cache(username: &str, token: &str, ttl_ms: i64) -> std::io::Result<()> {
    let cache_file = cache_file_path();

    let now = chrono::Local::now().timestamp_millis();
    let cache = TokenCache {
        username: username.to_string(),
        token: token.to_string(),
        expires_at: now + ttl_ms,
        created_at: now,
    };

    let json = serde_json::to_string_pretty(&cache).unwrap();
    let mut file = fs::File::create(&cache_file)?;
    file.write_all(json.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&cache_file, fs::Permissions::from_mode(0o600)).ok();
    }

    Ok(())
}

pub fn get_cache_status() -> CacheStatus {
    let cache_file = cache_file_path();

    match fs::read_to_string(&cache_file) {
        Ok(text) => {
            if let Ok(cache) = serde_json::from_str::<TokenCache>(&text) {
                let now = chrono::Local::now().timestamp_millis();
                let state = if now < cache.expires_at {
                    CacheState::Valid
                } else {
                    CacheState::Expired
                };
                CacheStatus {
                    state,
                    username: Some(cache.username),
                    expires_at: Some(cache.expires_at),
                    parse_error: None,
                }
            } else {
                CacheStatus {
                    state: CacheState::Invalid,
                    username: None,
                    expires_at: None,
                    parse_error: Some("Failed to parse cache file".to_string()),
                }
            }
        }
        Err(_) => CacheStatus {
            state: CacheState::Missing,
            username: None,
            expires_at: None,
            parse_error: None,
        },
    }
}

pub struct CacheStatus {
    pub state: CacheState,
    pub username: Option<String>,
    pub expires_at: Option<i64>,
    pub parse_error: Option<String>,
}
