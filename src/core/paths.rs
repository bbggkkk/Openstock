use std::env;
use std::path::PathBuf;

const APP_DIR: &str = "openstock";

pub fn config_dir() -> PathBuf {
    if let Ok(value) = env::var("OPENSTOCK_CONFIG_DIR") {
        let value = value.trim();
        if !value.is_empty() {
            return PathBuf::from(value);
        }
    }

    home_dir()
        .map(|home| home.join(".config").join(APP_DIR))
        .unwrap_or_else(|| PathBuf::from(".").join(".config").join(APP_DIR))
}

pub fn env_file() -> PathBuf {
    config_dir().join(".env")
}

pub fn score_file() -> PathBuf {
    config_dir().join("scores.json")
}

pub fn cache_dir() -> PathBuf {
    config_dir().join("cache")
}

pub fn cache_namespace(path: &str) -> PathBuf {
    cache_dir().join(path)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_file_lives_under_config_dir() {
        let path = env_file();
        assert!(path.ends_with(".env"));
    }

    #[test]
    fn cache_dir_lives_under_config_dir() {
        let path = cache_namespace("universe/kind");
        assert!(path.ends_with("cache/universe/kind"));
    }

    #[test]
    fn score_file_lives_under_config_dir() {
        let path = score_file();
        assert!(path.ends_with("scores.json"));
    }
}
