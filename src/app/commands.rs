use std::{path::Path, process};

use crate::app::{
    config::{Config, InitStatus},
    error::MmemoResult,
    expand::HomeDir,
    path_utils::{config_dir, config_path},
};

pub fn init() -> MmemoResult<()> {
    match Config::init()? {
        InitStatus::Created => {
            println!("Initialized.");
            println!("Config: {}", config_path()?.display());
        }
        InitStatus::AlreadyInitialized => {
            println!("Already initialized.");
            println!("Config: {}", config_path()?.display());
        }
    }
    Ok(())
}

pub fn new(config: Config, title: Vec<String>) -> MmemoResult<()> {
    let mut filename = title.join("_");

    let extension = Path::new(&filename).extension();

    if extension.is_none() {
        filename = format!("{}.md", filename);
    }
    let memo_dir = config.memo_dir.expand_home()?;

    process::Command::new(config.editor)
        .current_dir(memo_dir)
        .arg(filename)
        .status()?;
    Ok(())
}

pub fn config(config: Config) -> MmemoResult<()> {
    process::Command::new(config.editor)
        .current_dir(config_dir()?)
        .arg("config.toml")
        .status()?;
    Ok(())
}
