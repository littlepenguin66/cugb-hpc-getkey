use base64::{engine::general_purpose, Engine as _};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};

const RSA_PUBLIC_KEY: &str = "MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBALaXEnbjI6fjy+t9W9AiO/KS0q+b/OZFS+7ykinLbiriUx9P8BcuuHnVbXNiZp5jW70eVGBtX4DhGUPzJa1YT/8CAwEAAQ==";

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

pub fn encrypt_password(password: &str) -> String {
    let pem = base64_to_pem(RSA_PUBLIC_KEY);
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
