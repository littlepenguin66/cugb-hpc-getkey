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
use login::{download_key, login, LoginConfig};
use std::io::{self, Write};

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

    if !cli.force {
        if let Some(cached_token) = read_cache(&config.username) {
            if cli.verbose {
                eprintln!("✓ Using cached token");
            }
            let _ = download_key(&cached_token, &logger_options);
            if cli.verbose {
                println!("Token: {}", cached_token);
            }
            return;
        }
    }

    match login(&config, &logger_options) {
        Ok(token) => {
            let _ = write_cache(&config.username, &token, 2 * 60 * 60 * 1000);
            let _ = download_key(&token, &logger_options);
            if cli.verbose {
                println!("Token: {}", token);
            }
        }
        Err(e) => {
            eprintln!("Login failed: {}", e);
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
