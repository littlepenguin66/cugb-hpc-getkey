mod cache;
mod cli;
mod crypto;
mod login;
mod paths;
mod session;
mod types;

use cache::{CacheState, get_cache_status, read_cache, write_cache};
use chrono::TimeZone;
use clap::Parser;
use cli::Cli;
use login::{LoginConfig, download_key, login};
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};

type DynError = Box<dyn Error>;

#[derive(Debug, PartialEq, Eq)]
struct DownloadFlowResult {
    token: String,
    source: TokenSource,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenSource {
    Cache,
    Login,
}

#[derive(Serialize)]
struct FileStatusOutput {
    path: String,
    exists: bool,
    mode: Option<String>,
    modified_at_ms: Option<i64>,
    modified_at: Option<String>,
    size_bytes: Option<u64>,
}

#[derive(Serialize)]
struct StatusOutput {
    ok: bool,
    cache_state: &'static str,
    cache_exists: bool,
    cache_valid: bool,
    cache_parse_error: Option<String>,
    username: Option<String>,
    expires_at_ms: Option<i64>,
    expires_at: Option<String>,
    remaining_seconds: Option<i64>,
    cache: FileStatusOutput,
    key: FileStatusOutput,
}

#[derive(Serialize)]
struct SuccessOutput {
    ok: bool,
    source: &'static str,
    cache_path: String,
    key_path: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct ErrorOutput {
    ok: bool,
    error: String,
    detail: Option<String>,
    hint: Option<String>,
}

impl TokenSource {
    fn as_str(&self) -> &'static str {
        match self {
            TokenSource::Cache => "cache",
            TokenSource::Login => "login",
        }
    }
}

impl CacheState {
    fn as_str(&self) -> &'static str {
        match self {
            CacheState::Missing => "missing",
            CacheState::Invalid => "invalid",
            CacheState::Expired => "expired",
            CacheState::Valid => "valid",
        }
    }

    fn exists(&self) -> bool {
        !matches!(self, CacheState::Missing)
    }

    fn is_valid(&self) -> bool {
        matches!(self, CacheState::Valid)
    }
}

fn prompt_username() -> Result<String, DynError> {
    print!("Username: ");
    io::stdout().flush()?;

    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    Ok(username.trim().to_string())
}

fn prompt_password() -> Result<String, DynError> {
    rpassword::prompt_password("Password: ")
        .map_err(|error| format!("Failed to read password: {error}").into())
}

fn read_password_from_stdin() -> Result<String, DynError> {
    let mut password = String::new();
    io::stdin().read_to_string(&mut password)?;

    while matches!(password.chars().last(), Some('\n' | '\r')) {
        password.pop();
    }

    if password.is_empty() {
        return Err("Password read from stdin was empty".into());
    }

    Ok(password)
}

fn resolve_credentials(cli: &Cli) -> Result<(String, String), DynError> {
    let username = match cli.username.clone().filter(|value| !value.is_empty()) {
        Some(username) => username,
        None if cli.password_stdin => {
            return Err(
                "Username is required when using --password-stdin. Pass --username or set HPC_USERNAME"
                    .into(),
            )
        }
        None => prompt_username()?,
    };

    let password = if cli.password_stdin {
        read_password_from_stdin()?
    } else {
        match cli.password.clone().filter(|value| !value.is_empty()) {
            Some(password) => password,
            None => prompt_password()?,
        }
    };

    Ok((username, password))
}

fn format_expires_at(timestamp_millis: i64) -> String {
    chrono::Local
        .timestamp_millis_opt(timestamp_millis)
        .single()
        .unwrap()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn format_system_time(system_time: std::time::SystemTime) -> (i64, String) {
    let local_time: chrono::DateTime<chrono::Local> = system_time.into();
    (
        local_time.timestamp_millis(),
        local_time.format("%Y-%m-%d %H:%M:%S").to_string(),
    )
}

fn format_remaining_seconds(seconds: i64) -> String {
    if seconds <= 0 {
        return "0s".to_string();
    }

    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 {
        parts.push(format!("{seconds}s"));
    }

    parts.join(" ")
}

fn build_file_status(path: std::path::PathBuf) -> FileStatusOutput {
    let path_string = path.display().to_string();

    match fs::metadata(&path) {
        Ok(metadata) => {
            let (modified_at_ms, modified_at) = metadata
                .modified()
                .ok()
                .map(format_system_time)
                .map_or((None, None), |(ms, text)| (Some(ms), Some(text)));

            FileStatusOutput {
                path: path_string,
                exists: true,
                mode: file_mode(&metadata),
                modified_at_ms,
                modified_at,
                size_bytes: Some(metadata.len()),
            }
        }
        Err(_) => FileStatusOutput {
            path: path_string,
            exists: false,
            mode: None,
            modified_at_ms: None,
            modified_at: None,
            size_bytes: None,
        },
    }
}

#[cfg(unix)]
fn file_mode(metadata: &fs::Metadata) -> Option<String> {
    use std::os::unix::fs::PermissionsExt;

    Some(format!("{:03o}", metadata.permissions().mode() & 0o777))
}

#[cfg(not(unix))]
fn file_mode(_: &fs::Metadata) -> Option<String> {
    None
}

fn build_status_output() -> StatusOutput {
    let cache_status = get_cache_status();
    let cache = build_file_status(paths::cache_file_path());
    let key = build_file_status(paths::key_file_path());
    let now = chrono::Local::now().timestamp_millis();
    let remaining_seconds = cache_status
        .expires_at
        .map(|expires_at| ((expires_at - now).max(0)) / 1_000);

    StatusOutput {
        ok: true,
        cache_state: cache_status.state.as_str(),
        cache_exists: cache_status.state.exists(),
        cache_valid: cache_status.state.is_valid(),
        cache_parse_error: cache_status.parse_error,
        username: cache_status.username,
        expires_at_ms: cache_status.expires_at,
        expires_at: cache_status.expires_at.map(format_expires_at),
        remaining_seconds,
        cache,
        key,
    }
}

fn print_json<T: Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

fn print_status(output: &StatusOutput) {
    println!("Cache: {}", output.cache_state);
    println!("Cache path: {}", output.cache.path);
    println!(
        "Cache file: {}",
        if output.cache.exists { "present" } else { "missing" }
    );

    if let Some(mode) = &output.cache.mode {
        println!("Cache mode: {mode}");
    }
    if let Some(modified_at) = &output.cache.modified_at {
        println!("Cache modified: {modified_at}");
    }
    if let Some(size_bytes) = output.cache.size_bytes {
        println!("Cache size: {size_bytes} bytes");
    }
    if let Some(username) = &output.username {
        println!("Username: {username}");
    }
    if let Some(expires_at) = &output.expires_at {
        println!("Expires: {expires_at}");
    }
    if let Some(remaining_seconds) = output.remaining_seconds {
        let remaining = if output.cache_valid {
            format_remaining_seconds(remaining_seconds)
        } else {
            "expired".to_string()
        };
        println!("Remaining: {remaining}");
    }
    if let Some(parse_error) = &output.cache_parse_error {
        println!("Cache error: {parse_error}");
    }

    println!("Key path: {}", output.key.path);
    println!(
        "Key file: {}",
        if output.key.exists { "present" } else { "missing" }
    );

    if let Some(mode) = &output.key.mode {
        println!("Key mode: {mode}");
    }
    if let Some(modified_at) = &output.key.modified_at {
        println!("Key modified: {modified_at}");
    }
    if let Some(size_bytes) = output.key.size_bytes {
        println!("Key size: {size_bytes} bytes");
    }
}

fn handle_status(cli: &Cli) {
    let output = build_status_output();

    if cli.json {
        print_json(&output);
    } else {
        print_status(&output);
    }
}

fn handle_success(cli: &Cli, result: &DownloadFlowResult) {
    if cli.json {
        print_json(&SuccessOutput {
            ok: true,
            source: result.source.as_str(),
            cache_path: paths::cache_file_path().display().to_string(),
            key_path: paths::key_file_path().display().to_string(),
            token: cli.print_token.then(|| result.token.clone()),
        });
        return;
    }

    if cli.verbose && result.source == TokenSource::Cache {
        eprintln!("✓ Using cached token");
    }

    if cli.print_token {
        println!("Token: {}", result.token);
    }
}

fn classify_error(error: &dyn Error) -> (String, Option<String>) {
    let raw = error.to_string();
    let lower = raw.to_ascii_lowercase();

    if raw.starts_with("Failed to read password")
        || raw.starts_with("Password read from stdin was empty")
        || raw.starts_with("Username is required when using --password-stdin")
        || raw.starts_with("Failed to get execution token")
        || raw.starts_with("Failed to get token")
        || raw.starts_with("Failed to get private key")
        || raw.starts_with("Login failed, status:")
    {
        return (raw, None);
    }

    if lower.contains("dns failed") || lower.contains("failed to lookup address information") {
        return (
            "network error: failed to resolve hpc.cugb.edu.cn".to_string(),
            Some(raw),
        );
    }

    if lower.contains("timed out") {
        return ("network error: request timed out".to_string(), Some(raw));
    }

    if lower.contains("tls") || lower.contains("ssl") || lower.contains("certificate") {
        return (
            "network error: tls handshake failed".to_string(),
            Some(raw),
        );
    }

    if lower.contains("connection refused")
        || lower.contains("connection reset")
        || lower.contains("network is unreachable")
        || lower.contains("broken pipe")
    {
        return (
            "network error: connection to the hpc service failed".to_string(),
            Some(raw),
        );
    }

    if raw.starts_with("https://") || raw.starts_with("http://") {
        return (
            "network error: request to the hpc service failed".to_string(),
            Some(raw),
        );
    }

    if lower.contains("failed to determine home directory") {
        return (
            "local error: could not determine home directory".to_string(),
            Some(raw),
        );
    }

    if lower.contains("permission denied") {
        return (
            "local error: permission denied while writing local files".to_string(),
            Some(raw),
        );
    }

    (raw, None)
}

fn handle_error(cli: &Cli, error: &dyn Error) -> ! {
    let (message, detail) = classify_error(error);
    let hint = (!cli.verbose).then_some("use --verbose for step logs".to_string());

    if cli.json {
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&ErrorOutput {
                ok: false,
                error: message,
                detail,
                hint,
            })
            .unwrap()
        );
    } else {
        eprintln!("error: {message}");
        if cli.verbose {
            if let Some(detail) = &detail {
                eprintln!("detail: {detail}");
            }
        } else if let Some(hint) = hint {
            eprintln!("hint: {hint}");
        }
    }

    std::process::exit(1);
}

fn execute_download_flow<Download, LoginFn, CacheWriter>(
    force: bool,
    cached_token: Option<String>,
    mut download: Download,
    mut login_fn: LoginFn,
    mut cache_writer: CacheWriter,
) -> Result<DownloadFlowResult, DynError>
where
    Download: FnMut(&str) -> Result<(), DynError>,
    LoginFn: FnMut() -> Result<String, DynError>,
    CacheWriter: FnMut(&str) -> Result<(), DynError>,
{
    if !force
        && let Some(token) = cached_token
        && download(&token).is_ok()
    {
        return Ok(DownloadFlowResult {
            token,
            source: TokenSource::Cache,
        });
    }

    let token = login_fn()?;
    download(&token)?;
    cache_writer(&token)?;

    Ok(DownloadFlowResult {
        token,
        source: TokenSource::Login,
    })
}

fn main() {
    let cli = Cli::parse();

    if cli.status {
        handle_status(&cli);
        return;
    }

    let (username, password) = match resolve_credentials(&cli) {
        Ok(credentials) => credentials,
        Err(error) => handle_error(&cli, error.as_ref()),
    };

    let config = LoginConfig {
        username,
        password,
        service: "https://hpc.cugb.edu.cn/ac/api/auth/loginSsoRedirect.action".to_string(),
    };

    let logger_options = cli.get_logger_options();

    match execute_download_flow(
        cli.force,
        read_cache(&config.username),
        |token| download_key(token, &logger_options),
        || login(&config, &logger_options),
        |token| write_cache(&config.username, token, 2 * 60 * 60 * 1000).map_err(Into::into),
    ) {
        Ok(result) => handle_success(&cli, &result),
        Err(error) => handle_error(&cli, error.as_ref()),
    }
}
