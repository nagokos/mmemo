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
    template::load_template,
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

pub fn new(config: &Config, title: &str) -> MmemoResult<()> {
    let mut filename = title.replace(" ", "_");

    let extension = Path::new(&filename).extension();

    if extension.is_none() {
        filename = format!("{}.md", filename);
    }

    // TODO: templateあるなしでファイルの作成の有無が変わってる
    let file_path = config.memo_dir.expand_home()?.join(&filename);

    if !file_path.exists()
        && let Some(path) = config.memo_template.clone()
    {
        let file = File::open(path.expand_home()?)?;
        let template = load_template(title, file)?;
        fs::write(&file_path, template)?;
    }

    process::Command::new(&config.editor)
        .arg(&file_path)
        .status()?;

    Ok(())
}

pub fn edit(config: &Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;
    let files = dir_files(&memo_dir)?;

    let selector = selector::selector_select(&config.selector);
    if let Some(result) = selector.select(files)? {
        process::Command::new(&config.editor)
            .current_dir(memo_dir)
            .arg(result)
            .status()?;
    }

    Ok(())
}

pub fn delete(config: &Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;
    let files = dir_files(&memo_dir)?;

    let selector = selector::selector_select(&config.selector);
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

pub fn list(config: &Config) -> MmemoResult<()> {
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

pub fn view(config: &Config) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;
    let files = dir_files(&memo_dir)?;

    let selector = selector::selector_select(&config.selector);
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

pub fn grep(config: &Config, rest: &[String]) -> MmemoResult<()> {
    let memo_dir = config.memo_dir.expand_home()?;

    match config.grep {
        GrepKind::Builtin => {
            if rest.iter().any(|s| s.is_empty()) {
                return Err(MmemoError::InvalidArgs {
                    message: "empty pattern".into(),
                });
            }

            if rest.iter().any(|a| a.starts_with('-')) {
                return Err(MmemoError::InvalidArgs {
                    message: "builtin grep does not support options (set grep = \"ripgrep\")"
                        .into(),
                });
            }

            let needles: Vec<String> = rest
                .iter()
                .filter(|s| !s.starts_with('-'))
                .cloned()
                .collect();

            let files = dir_files(&memo_dir)?;

            for file in files {
                let f = File::open(memo_dir.join(&file))?;
                let reader = BufReader::new(f);

                let lines: Vec<_> = reader
                    .lines()
                    .enumerate()
                    .filter_map(|(i, line)| {
                        let line = line.ok()?;
                        rest.iter().all(|r| line.contains(r)).then(|| (i + 1, line))
                    })
                    .collect();

                if !lines.is_empty() {
                    println!("{}", file);

                    for (row, line) in lines {
                        println!("{}: {}", row, highlight_all(&line, &needles));
                    }
                    println!();
                }
            }
        }
        GrepKind::Rg => {
            process::Command::new("rg")
                .current_dir(memo_dir)
                .args(rest)
                .status()?;
        }
    }

    Ok(())
}

fn highlight_all(line: &str, needles: &[String]) -> String {
    let mut ranges: Vec<(usize, usize)> = Vec::new();

    for n in needles {
        if n.is_empty() {
            continue;
        }
        for (start, _) in line.match_indices(n) {
            ranges.push((start, start + n.len()));
        }
    }

    if ranges.is_empty() {
        return line.to_string();
    }

    // 位置順に並べて、重なりはマージ（“赤くする領域の和集合”にする）
    ranges.sort_by_key(|(s, e)| (*s, *e));
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in ranges {
        match merged.last_mut() {
            Some((ls, le)) if s <= *le => *le = (*le).max(e),
            _ => merged.push((s, e)),
        }
    }

    let mut out = String::new();
    let mut cur = 0;
    for (s, e) in merged {
        out.push_str(&line[cur..s]);
        out.push_str("\x1b[31m");
        out.push_str(&line[s..e]);
        out.push_str("\x1b[0m");
        cur = e;
    }
    out.push_str(&line[cur..]);
    out
}

pub fn config(config: &Config) -> MmemoResult<()> {
    process::Command::new(&config.editor)
        .current_dir(config_dir()?)
        .arg("config.toml")
        .status()?;
    Ok(())
}

const HELP: &str = r#"mmemo - A simple CLI memo management tool

USAGE:
    mmemo <command> [args]

COMMANDS:
    init, i              Initialize configuration and create config.toml
    new, n <title...>    Create a new memo (spaces are allowed)
    list, l              List all memos
    edit, e              Select and edit a memo
    view, v              Select and view a memo
    grep, g <pat...>     Search memos
    delete, d            Select and delete a memo
    config, c            Open config.toml in your editor

GLOBAL OPTIONS:
    -h, --help           Show help
    -v, --version        Show version

NOTES:
    The behavior of some commands depends on config.toml.

    selector (used by edit/view/delete):
      - selector = "builtin"  : use builtin selector
      - selector = "fzf"      : use external "fzf"
      - selector = "skim"     : use external "sk" (skim)

    view:
      - viewer = "builtin"    : render markdown in terminal
      - viewer = "glow"       : use external "glow" command

    grep:
      - grep = "builtin"      : simple AND search (all patterns must appear in the line)
      - grep = "ripgrep"      : pass arguments to "rg" as-is
        (If a pattern starts with '-', use: mmemo grep -e "-foo")

    If an external command is not found, switch the corresponding setting to "builtin".

EXAMPLES:
    mmemo init
    mmemo new my memo
    mmemo list

    mmemo edit                         # selector depends on config.toml
    mmemo view                         # selector/viewer depend on config.toml

    mmemo grep todo                    # search "todo"
    mmemo grep foo bar                 # AND search (builtin): both "foo" and "bar"
    mmemo grep -n todo                 # ripgrep only (grep = "ripgrep")
    mmemo grep -e "-foo"               # ripgrep: pattern starting with '-'

    mmemo config
    mmemo --help
    mmemo --version
"#;

pub fn help() {
    println!("{HELP}")
}
pub fn version() {
    println!("mmemo {}", env!("CARGO_PKG_VERSION"));
}
