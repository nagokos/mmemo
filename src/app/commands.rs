use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
    process,
};

use crate::app::{
    config::{Config, InitStatus},
    error::MmemoResult,
    expand::HomeDir,
    path_utils::{config_dir, config_path},
    selector,
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

pub fn edit(config: Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;
    let files = dir_files(&memo_dir)?;

    let selector = selector::selector_select(config.selector);
    if let Some(result) = selector.select(files)? {
        process::Command::new(config.editor)
            .current_dir(memo_dir)
            .arg(result)
            .status()?;
    }

    Ok(())
}

pub fn delete(config: Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;
    let files = dir_files(&memo_dir)?;

    let selector = selector::selector_select(config.selector);
    if let Some(result) = selector.select(files)? {
        process::Command::new("rm")
            .current_dir(memo_dir)
            .arg(result)
            .status()?;
    }

    Ok(())
}

pub fn dir_files(dir: &Path) -> MmemoResult<Vec<String>> {
    let mut files = Vec::new();
    let mut cd = |entry: &DirEntry| {
        files.push(
            entry
                .path()
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );
    };
    visit_dirs(dir, &mut cd)?;

    Ok(files)
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
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
