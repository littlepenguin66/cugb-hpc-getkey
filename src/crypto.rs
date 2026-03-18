use base64::{engine::general_purpose, Engine as _};
use regex_lite::Regex;
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};

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
    let body = res.into_string()?;

    let re = Regex::new(r#"var key = '([^']+)'"#).unwrap();
    if let Some(caps) = re.captures(&body) {
        Ok(caps[1].to_string())
    } else {
        Err("Failed to extract public key from login.js".into())
    }
}

pub fn encrypt_password(password: &str) -> String {
    let public_key_base64 = fetch_public_key().unwrap_or_else(|_| {
        eprintln!("Warning: Using fallback public key");
        DEFAULT_PUBLIC_KEY.to_string()
    });

    let pem = base64_to_pem(&public_key_base64);
    let public_key = RsaPublicKey::from_public_key_pem(&pem).unwrap();

    let encrypted = public_key
        .encrypt(
            &mut rand::thread_rng(),
            Pkcs1v15Encrypt,
            password.as_bytes(),
        )
        .unwrap();

    general_purpose::STANDARD.encode(&encrypted)
}
