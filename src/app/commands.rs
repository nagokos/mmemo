use std::{
    fs::{self, DirEntry, File},
    io::{self},
    path::Path,
    process,
};

use chrono::{DateTime, Datelike, Utc};

use crate::app::{
    config::{Config, InitStatus},
    error::MmemoResult,
    expand::HomeDir,
    path_utils::{config_dir, config_path},
    selector,
    template::Template,
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

    let file_path = config.memo_dir.expand_home()?.join(&filename);

    if !file_path.exists()
        && let Some(path) = config.memo_template
    {
        let file = File::open(path.expand_home()?)?;
        let template = Template::load(&title.join(" "), file)?;
        fs::write(&file_path, template)?;
    }

    process::Command::new(config.editor)
        .arg(&file_path)
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
        let file = entry
            .path()
            .strip_prefix(dir)
            .unwrap()
            .to_string_lossy()
            .to_string();

        files.push(file);
    };
    visit_dirs(dir, &mut cd)?;

    Ok(files)
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_name().to_string_lossy().starts_with(".") {
                continue;
            }

            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub fn list(config: Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;

    println!("Memos in {}:\n", memo_dir.display());

    let files = dir_files(&memo_dir)?;

    for file in &files {
        let file_path = memo_dir.join(file);
        let f = File::open(file_path)?;

        let date_time: DateTime<Utc> = f.metadata()?.created()?.into();
        let created_time = format!(
            "{}-{:02}-{:02}",
            date_time.year(),
            date_time.month(),
            date_time.day()
        );

        println!("{:<width$} {}", file, created_time, width = 40)
    }

    println!("\nTotal: {} memos", files.len());

    Ok(())
}

pub fn config(config: Config) -> MmemoResult<()> {
    process::Command::new(config.editor)
        .current_dir(config_dir()?)
        .arg("config.toml")
        .status()?;
    Ok(())
}
