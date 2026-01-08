use std::{
    fs::{self, DirEntry, File},
    io::{self, BufRead, BufReader, Read},
    path::Path,
    process,
};

use chrono::{DateTime, Datelike, Utc};
use termimad::{Alignment, MadSkin};

use crate::app::{
    config::{Config, GrepKind, InitStatus, ViewerKind},
    error::{MmemoError, MmemoResult},
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

    // TODO: templateあるなしでファイルの作成の有無が変わってる
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

pub fn view(config: Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;
    let files = dir_files(&memo_dir)?;

    let selector = selector::selector_select(config.selector);
    if let Some(result) = selector.select(files)? {
        match config.viewer {
            ViewerKind::Builtin => {
                let mut file = File::open(memo_dir.join(result))?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                let mut skin = MadSkin::default();
                for header in &mut skin.headers {
                    header.align = Alignment::Left;
                }
                skin.print_text(&buf);
            }
            ViewerKind::Glow => {
                process::Command::new("glow")
                    .current_dir(memo_dir)
                    .arg(result)
                    .status()
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            MmemoError::Config {
                                message: "glow not found. Install glow or use builtin viewer"
                                    .to_string(),
                            }
                        } else {
                            e.into()
                        }
                    })?;
            }
        }
    }
    Ok(())
}

pub fn grep(config: Config, options: Vec<String>, target: String) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;

    match config.grep {
        GrepKind::Builtin => {
            let files = dir_files(&memo_dir)?;
            for file in files {
                let f = File::open(memo_dir.join(&file))?;
                let reader = BufReader::new(f);

                let lines: Vec<_> = reader
                    .lines()
                    .enumerate()
                    .filter_map(|(i, line)| {
                        let line = line.ok()?;
                        line.contains(&target).then(|| (i + 1, line))
                    })
                    .collect();

                if !lines.is_empty() {
                    println!("{}", file);
                    for (row, line) in lines {
                        let highlighted =
                            line.replace(&target, &format!("\x1b[31m{}\x1b[0m", target));
                        println!("{}: {}", row, highlighted);
                    }
                    println!();
                }
            }
        }
        GrepKind::Rg => {
            process::Command::new("rg")
                .current_dir(memo_dir)
                .args(options)
                .arg(target)
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
