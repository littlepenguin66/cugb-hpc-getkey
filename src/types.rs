use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LoginConfig {
    pub username: String,
    pub password: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCache {
    pub username: String,
    pub token: String,
    pub expires_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct LoggerOptions {
    pub quiet: bool,
    pub verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub code: String,
    pub data: TokenData,
    #[serde(default)]
    pub msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    #[serde(rename = "tokenList", default)]
    pub token_list: Vec<TokenItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenItem {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadKeyResponse {
    pub code: String,
    pub data: Option<String>,
    pub msg: Option<String>,
}
