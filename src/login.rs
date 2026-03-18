use crate::crypto::encrypt_password;
use crate::session::get_cookie_string;
pub use crate::types::LoginConfig;
use crate::types::{DownloadKeyResponse, LoggerOptions, TokenResponse};
use regex_lite::Regex;
use std::collections::HashMap;
use ureq::Agent;

const SERVICE: &str = "https://hpc.cugb.edu.cn/ac/api/auth/loginSsoRedirect.action";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct Logger {
    options: LoggerOptions,
}

impl Logger {
    pub fn new(options: LoggerOptions) -> Self {
        Logger { options }
    }

    pub fn info(&self, message: &str) {
        if !self.options.quiet {
            println!("{}", message);
        }
    }

    pub fn debug(&self, message: &str) {
        if self.options.verbose {
            eprintln!("{}", message);
        }
    }
}

fn log_error(message: &str) {
    eprintln!("{}", message);
}

pub fn login(
    config: &LoginConfig,
    logger_options: &LoggerOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let logger = Logger::new(logger_options.clone());
    let mut cookies: HashMap<String, String> = HashMap::new();
    let agent = Agent::new();

    logger.debug("→ GET login page");
    let login_page_url = format!(
        "https://hpc.cugb.edu.cn/sso/login?service={}&t={}",
        urlencoding::encode(&config.service),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let login_page_res = agent.get(&login_page_url).call()?;
    update_cookies_from_response(&login_page_res, &mut cookies);
    let login_page_html = login_page_res.into_string()?;
    let execution = extract_execution_token(&login_page_html)?;

    logger.debug("→ Encrypt password");
    let encrypted_password = encrypt_password(&config.password);

    logger.debug("→ POST credentials");
    let login_body = build_login_body(&config.username, &encrypted_password, execution);

    let login_res = agent
        .post(&login_page_url)
        .set("Cookie", &get_cookie_string(&cookies))
        .set("Origin", "https://hpc.cugb.edu.cn")
        .set("Referer", &login_page_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("User-Agent", USER_AGENT)
        .send_string(&login_body)?;

    update_cookies_from_response(&login_res, &mut cookies);
    handle_login_response(login_res, &agent, &mut cookies)?;

    logger.debug("→ Get JWT token");
    let token = fetch_jwt_token(&agent, &cookies)?;
    Ok(token)
}

fn extract_execution_token(html: &str) -> Result<&str, &'static str> {
    let execution_re = Regex::new(r#"name="execution"\s+value="([^"]+)""#).unwrap();
    execution_re
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .ok_or("Failed to get execution token")
}

fn build_login_body(username: &str, password: &str, execution: &str) -> String {
    format!(
        "username={}&password={}&encrypted={}&mode={}&captcha={}&execution={}&_eventId={}&geolocation={}&submit={}",
        urlencoding::encode(username),
        urlencoding::encode(password),
        urlencoding::encode("true"),
        urlencoding::encode("0"),
        urlencoding::encode(""),
        urlencoding::encode(execution),
        urlencoding::encode("submit"),
        urlencoding::encode(""),
        urlencoding::encode("登录")
    )
}

fn handle_login_response(
    login_res: ureq::Response,
    agent: &Agent,
    cookies: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if login_res.status() == 302 {
        if let Some(location) = login_res.header("Location") {
            if let Some(ticket) = extract_ticket(location) {
                let sso_res = agent
                    .get(&format!("{}?ticket={}", SERVICE, ticket))
                    .set("Cookie", &get_cookie_string(cookies))
                    .call()?;
                update_cookies_from_response(&sso_res, cookies);
            }
        }
    } else if login_res.status() == 200 {
        let login_body = login_res.into_string()?;
        if let Some(redirect_url) = extract_redirect_url(&login_body) {
            let redirect_res = agent
                .get(redirect_url)
                .set("Cookie", &get_cookie_string(cookies))
                .call()?;
            update_cookies_from_response(&redirect_res, cookies);

            if redirect_res.status() == 302 {
                if let Some(next_location) = redirect_res.header("Location") {
                    let next_res = agent
                        .get(next_location)
                        .set("Cookie", &get_cookie_string(cookies))
                        .call()?;
                    update_cookies_from_response(&next_res, cookies);
                }
            }
        }
    } else {
        return Err(format!("Login failed, status code: {}", login_res.status()).into());
    }
    Ok(())
}

fn extract_ticket(location: &str) -> Option<&str> {
    location
        .split('?')
        .nth(1)?
        .split('&')
        .find(|p| p.starts_with("ticket="))
        .and_then(|t| t.strip_prefix("ticket="))
}

fn extract_redirect_url(html: &str) -> Option<&str> {
    let redirect_re = Regex::new(r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#).unwrap();
    redirect_re
        .captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
}

fn update_cookies_from_response(res: &ureq::Response, cookies: &mut HashMap<String, String>) {
    for cookie in res.all("set-cookie") {
        if let Some(eq) = cookie.split(';').next().and_then(|s| s.find('=')) {
            let key = cookie[..eq].to_string();
            let value = cookie[eq + 1..].to_string();
            cookies.insert(key, value);
        }
    }
}

fn fetch_jwt_token(
    agent: &Agent,
    cookies: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let token_url = "https://hpc.cugb.edu.cn/ac/api/user/getCurrentUserInfo.action?includeToken=true&refresh=true";
    let token_res = agent
        .get(token_url)
        .set("Cookie", &get_cookie_string(cookies))
        .call()?;

    let token_text = token_res.into_string()?;
    let token_data: TokenResponse = serde_json::from_str(&token_text)?;

    if token_data.code != "0" || token_data.data.token_list.is_empty() {
        return Err("Failed to get token".into());
    }

    Ok(token_data.data.token_list[0].token.clone())
}

pub fn download_key(
    jwt_token: &str,
    logger_options: &LoggerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new(logger_options.clone());

    logger.debug("→ Download key");
    let api_url = "https://gridview.cugb.edu.cn:6081/sothisai/api/eshell/action/downloadkey";
    let api_res = ureq::get(api_url)
        .set("token", jwt_token)
        .set("Accept", "application/json")
        .call()?;

    let api_data: DownloadKeyResponse = api_res.into_json()?;

    if api_data.code == "0" && api_data.data.is_some() {
        let key_path = format!("{}/.hpckey", dirs::home_dir().unwrap().display());
        std::fs::write(&key_path, api_data.data.as_ref().unwrap())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600)).ok();
        }

        logger.info(&format!("Private key saved to: {}", key_path));
    } else {
        log_error(&format!(
            "Failed to get private key: {}",
            api_data.msg.unwrap_or_default()
        ));
    }

    Ok(())
}
