use std::{env, path::PathBuf};

use crate::app::error::{MmemoError, MmemoResult};

// TODO: Result返していいのかと、app/error.rsだけど依存の関係性src/error.rsの方がいいのか
pub fn home_dir() -> MmemoResult<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(MmemoError::EnvVarMissing { key: "HOME" })
}

pub fn config_dir() -> MmemoResult<PathBuf> {
    Ok(home_dir()?.join(".config").join("mmemo"))
}

pub fn mmemo_dir() -> MmemoResult<PathBuf> {
    Ok(home_dir()?.join("mmemo"))
}

pub fn config_path() -> MmemoResult<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn template_path() -> MmemoResult<PathBuf> {
    Ok(config_dir()?.join("template.md"))
}
