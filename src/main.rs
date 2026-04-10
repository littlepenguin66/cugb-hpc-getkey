mod cache;
mod cli;
mod crypto;
mod login;
mod session;
mod types;

use cache::{get_cache_status, read_cache, write_cache};
use chrono::TimeZone;
use clap::Parser;
use cli::Cli;
use login::{LoginConfig, download_key, login};
use std::error::Error;
use std::io::{self, Write};

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

fn prompt_credentials() -> (String, String) {
    print!("Username: ");
    io::stdout().flush().unwrap();

    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();

    let password = rpassword::prompt_password("Password: ").unwrap();

    (username, password)
}

fn format_expires_at(timestamp_millis: i64) -> String {
    chrono::Local
        .timestamp_millis_opt(timestamp_millis)
        .single()
        .unwrap()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
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
        handle_status();
        return;
    }

    let mut username = cli.username.clone().filter(|s| !s.is_empty());
    let mut password = cli.password.clone().filter(|s| !s.is_empty());

    if username.is_none() || password.is_none() {
        let (u, p) = prompt_credentials();
        username = Some(u);
        password = Some(p);
    }

    let config = LoginConfig {
        username: username.unwrap(),
        password: password.unwrap(),
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
        Ok(result) => {
            if cli.verbose {
                if result.source == TokenSource::Cache {
                    eprintln!("✓ Using cached token");
                }
                println!("Token: {}", result.token);
            }
        }
        Err(e) => {
            eprintln!("Operation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_status() {
    let status = get_cache_status();
    if status.exists {
        let expires = format_expires_at(status.expires_at.unwrap());
        let validity = if status.valid { "valid" } else { "expired" };
        println!("Cache status: {}", validity);
        println!("Username: {}", status.username.unwrap());
        println!("Expires: {}", expires);
    } else {
        println!("No cache");
    }
}
