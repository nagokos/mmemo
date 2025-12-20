use std::{
    io::{Write, stdin, stdout},
    process,
};

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

pub fn new(config: Config, opt: Option<Vec<String>>) -> MmemoResult<()> {
    match opt {
        Some(title) => {
            let filename = format!("{}.md", title.join("_"));
            let memo_dir = config.memo_dir.expand_home()?;

            process::Command::new(config.editor)
                .current_dir(memo_dir)
                .arg(filename)
                .status()?;
        }
        None => {
            let mut title = String::new();
            print!("Title: ");
            stdout().flush()?;
            stdin().read_line(&mut title)?;

            let filename = format!("{}.md", title.trim().replace(" ", "_"));
            let memo_dir = config.memo_dir.expand_home()?;
            process::Command::new(config.editor)
                .current_dir(memo_dir)
                .arg(filename)
                .status()?;
        }
    }
    Ok(())
}

pub fn config(config: Config) -> MmemoResult<()> {
    process::Command::new(config.editor)
        .current_dir(config_dir()?)
        .arg("config.toml")
        .status()?;
    Ok(())
}
