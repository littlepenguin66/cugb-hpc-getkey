use crate::crypto::encrypt_password;
use crate::session::get_cookie_string;
pub use crate::types::LoginConfig;
use crate::types::{DownloadKeyResponse, LoggerOptions, TokenResponse};
use regex_lite::Regex;
use std::collections::HashMap;
use std::error::Error;
use ureq::{Agent, AgentBuilder};

const SERVICE: &str = "https://hpc.cugb.edu.cn/ac/api/auth/loginSsoRedirect.action";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "zh-CN,zh;q=0.9,en;q=0.8";

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

pub fn login(
    config: &LoginConfig,
    logger_options: &LoggerOptions,
) -> Result<String, Box<dyn Error>> {
    let logger = Logger::new(logger_options.clone());
    let mut cookies: HashMap<String, String> = HashMap::new();
    let agent = AgentBuilder::new().redirects(0).build();

    logger.debug("Step 1: Fetching login page...");
    let login_page_url = format!(
        "https://hpc.cugb.edu.cn/sso/login?service={}&t={}",
        urlencoding::encode(&config.service),
        timestamp()
    );

    let login_page_res = agent
        .get(&login_page_url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", ACCEPT)
        .set("Accept-Language", ACCEPT_LANGUAGE)
        .call()?;
    update_cookies(&login_page_res, &mut cookies);

    let login_page_html = login_page_res.into_string()?;
    let execution = extract_execution_token(&login_page_html)?;

    logger.debug("Step 2: Encrypting password...");
    let encrypted_password = encrypt_password(&config.password)?;

    logger.debug("Step 3: Sending login request...");
    let login_res = agent
        .post(&login_page_url)
        .set("Cookie", &get_cookie_string(&cookies))
        .set("Origin", "https://hpc.cugb.edu.cn")
        .set("Referer", &login_page_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("User-Agent", USER_AGENT)
        .set("Accept", ACCEPT)
        .set("Accept-Language", ACCEPT_LANGUAGE)
        .send_string(&build_login_body(
            &config.username,
            &encrypted_password,
            execution,
        ))?;

    let status = login_res.status();
    update_cookies(&login_res, &mut cookies);

    match status {
        302 => handle_ticket_redirect(&login_res, &agent, &mut cookies)?,
        200 => handle_js_redirect(login_res, &agent, &mut cookies)?,
        _ => return Err(format!("Login failed, status: {}", status).into()),
    }

    logger.debug("Step 4: Getting JWT token...");
    fetch_jwt_token(&agent, &cookies)
}

fn timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn handle_ticket_redirect(
    res: &ureq::Response,
    agent: &Agent,
    cookies: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let location = match res.header("Location").and_then(|l| extract_ticket(l)) {
        Some(t) => t,
        None => return Ok(()),
    };

    let sso_res = agent
        .get(&format!("{}?ticket={}", SERVICE, location))
        .set("Cookie", &get_cookie_string(cookies))
        .set("User-Agent", USER_AGENT)
        .set("Accept", ACCEPT)
        .call()?;
    update_cookies(&sso_res, cookies);

    if sso_res.status() == 302 {
        follow_redirect(&sso_res, agent, cookies)?;
    }
    Ok(())
}

fn handle_js_redirect(
    res: ureq::Response,
    agent: &Agent,
    cookies: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = res.into_string()?;
    let redirect_url = match extract_redirect_url(&body) {
        Some(u) => u,
        None => return Ok(()),
    };

    let redirect_res = agent
        .get(redirect_url)
        .set("Cookie", &get_cookie_string(cookies))
        .set("User-Agent", USER_AGENT)
        .set("Accept", ACCEPT)
        .call()?;
    update_cookies(&redirect_res, cookies);

    if redirect_res.status() == 302 {
        follow_redirect(&redirect_res, agent, cookies)?;
    }
    Ok(())
}

fn follow_redirect(
    res: &ureq::Response,
    agent: &Agent,
    cookies: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(location) = res.header("Location") {
        let next_res = agent
            .get(&resolve_url(location))
            .set("Cookie", &get_cookie_string(cookies))
            .set("User-Agent", USER_AGENT)
            .set("Accept", ACCEPT)
            .call()?;
        update_cookies(&next_res, cookies);
    }
    Ok(())
}

fn resolve_url(url: &str) -> String {
    if url.starts_with("http") {
        url.to_string()
    } else {
        format!("https://hpc.cugb.edu.cn{}", url)
    }
}

fn extract_execution_token(html: &str) -> Result<&str, &'static str> {
    let re = Regex::new(r#"name="execution"\s+value="([^"]+)""#).unwrap();
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .ok_or("Failed to get execution token")
}

fn build_login_body(username: &str, password: &str, execution: &str) -> String {
    format!(
        "username={}&password={}&encrypted=true&mode=0&captcha=&execution={}&_eventId=submit&geolocation=&submit={}",
        urlencoding::encode(username),
        urlencoding::encode(password),
        urlencoding::encode(execution),
        urlencoding::encode("登录")
    )
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
    let re = Regex::new(r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#).unwrap();
    re.captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
}

fn update_cookies(res: &ureq::Response, cookies: &mut HashMap<String, String>) {
    for cookie in res.all("set-cookie") {
        let pair = cookie.split(';').next().unwrap_or("");
        if let Some(pos) = pair.find('=') {
            cookies.insert(pair[..pos].to_string(), pair[pos + 1..].to_string());
        }
    }
}

fn fetch_jwt_token(
    agent: &Agent,
    cookies: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let res = agent
        .get("https://hpc.cugb.edu.cn/ac/api/user/getCurrentUserInfo.action?includeToken=true&refresh=true")
        .set("Cookie", &get_cookie_string(cookies))
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()?;

    let data: TokenResponse = serde_json::from_str(&res.into_string()?)?;

    if data.code != "0" || data.data.token_list.is_empty() {
        return Err("Failed to get token".into());
    }

    Ok(data.data.token_list[0].token.clone())
}

fn extract_private_key(response: DownloadKeyResponse) -> Result<String, Box<dyn Error>> {
    match (response.code.as_str(), response.data, response.msg) {
        ("0", Some(private_key), _) => Ok(private_key),
        (_, _, Some(message)) if !message.is_empty() => {
            Err(format!("Failed to get private key: {}", message).into())
        }
        _ => Err("Failed to get private key".into()),
    }
}

pub fn download_key(jwt_token: &str, logger_options: &LoggerOptions) -> Result<(), Box<dyn Error>> {
    let logger = Logger::new(logger_options.clone());

    logger.debug("Step 5: Downloading private key...");
    let res = ureq::get("https://gridview.cugb.edu.cn:6081/sothisai/api/eshell/action/downloadkey")
        .set("token", jwt_token)
        .set("Accept", "application/json")
        .call()?;

    let private_key = extract_private_key(res.into_json()?)?;
    let path = dirs::home_dir()
        .ok_or("Failed to determine home directory")?
        .join(".hpckey");
    std::fs::write(&path, private_key)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    logger.info(&format!("Private key saved to: {}", path.display()));

    Ok(())
}
