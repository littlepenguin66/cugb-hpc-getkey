use std::path::PathBuf;

fn home_dir_or_tilde() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
}

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn cache_file_path() -> PathBuf {
    home_dir_or_tilde().join(".hpc-login-cache.json")
}

pub fn key_file_path() -> PathBuf {
    home_dir_or_tilde().join(".hpckey")
}
