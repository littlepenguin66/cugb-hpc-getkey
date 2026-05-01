use crate::types::LoggerOptions;
use base64::{Engine as _, engine::general_purpose};
use regex_lite::Regex;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs8::DecodePublicKey};
use std::error::Error;

const LOGIN_JS_URL: &str =
    "https://hpc.cugb.edu.cn/sso/themes/sso/js/login-744fe89e6ff1efcab5fff7e1668641b0.js";
const DEFAULT_PUBLIC_KEY: &str = "MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBALaXEnbjI6fjy+t9W9AiO/KS0q+b/OZFS+7ykinLbiriUx9P8BcuuHnVbXNiZp5jW70eVGBtX4DhGUPzJa1YT/8CAwEAAQ==";

fn base64_to_pem(base64_key: &str) -> String {
    let lines: Vec<&str> = base64_key
        .as_bytes()
        .chunks(64)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        lines.join("\n")
    )
}

fn fetch_public_key() -> Result<String, Box<dyn std::error::Error>> {
    let res = ureq::get(LOGIN_JS_URL).call()?;
    let body = res.into_body().read_to_string()?;

    let re = Regex::new(r#"var key = '([^']+)'"#).unwrap();
    if let Some(caps) = re.captures(&body) {
        Ok(caps[1].to_string())
    } else {
        Err("Failed to extract public key from login.js".into())
    }
}

fn encrypt_password_with_public_key(
    password: &str,
    public_key_base64: &str,
) -> Result<String, Box<dyn Error>> {
    let pem = base64_to_pem(public_key_base64);
    let public_key = RsaPublicKey::from_public_key_pem(&pem)?;
    let encrypted = public_key.encrypt(
        &mut rsa::rand_core::OsRng,
        Pkcs1v15Encrypt,
        password.as_bytes(),
    )?;

    Ok(general_purpose::STANDARD.encode(&encrypted))
}

pub fn encrypt_password(password: &str, logger_options: &LoggerOptions) -> Result<String, Box<dyn Error>> {
    let public_key_base64 = match fetch_public_key() {
        Ok(key) => key,
        Err(error) => {
            if !logger_options.json {
                eprintln!("Warning: Failed to fetch public key: {error}. Using fallback public key");
            }
            DEFAULT_PUBLIC_KEY.to_string()
        }
    };

    encrypt_password_with_public_key(password, &public_key_base64)
}
