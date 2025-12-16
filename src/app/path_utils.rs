use std::{env, path::PathBuf};

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME environment variable not set")
}

pub fn config_dir() -> PathBuf {
    home_dir().join(".config").join("mmemo")
}

pub fn mmemo_dir() -> PathBuf {
    home_dir().join("mmemo")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn template_path() -> PathBuf {
    config_dir().join("template.md")
}
