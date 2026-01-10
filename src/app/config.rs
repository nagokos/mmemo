use std::{
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use toml::Table;

use crate::app::{
    error::{MmemoError, MmemoResult},
    expand::HomeDir,
    path_utils::{config_dir, config_path, mmemo_dir, template_path},
};

struct Lexer {
    contents: String,
}

impl Lexer {
    fn new(mut file: File) -> MmemoResult<Self> {
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(Self { contents: buf })
    }
    fn tokenize(&self) -> MmemoResult<Vec<Token>> {
        let mut tokens = Vec::new();

        let table = toml::from_str::<Table>(&self.contents).map_err(|_| MmemoError::Config {
            message: "Please check the configuration settings.".to_string(),
        })?;

        for (key, val) in &table {
            let key: ConfigKey = key.parse().map_err(|_| MmemoError::Parse {
                message: key.to_string(),
            })?;
            let value = val
                .as_str()
                .ok_or(MmemoError::Parse {
                    message: format!("{} must be a string", key),
                })?
                .to_string();

            let token = Token { key, value };
            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[derive(Debug)]
struct Token {
    key: ConfigKey,
    value: String,
}

#[derive(Debug)]
pub enum SelectorKind {
    Builtin,
    Fzf,
    Skim,
}

pub struct ParseSelectorKindError;

impl FromStr for SelectorKind {
    type Err = ParseSelectorKindError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "builtin" => Ok(SelectorKind::Builtin),
            "fzf" => Ok(SelectorKind::Fzf),
            "skim" => Ok(SelectorKind::Skim),
            _ => Err(ParseSelectorKindError),
        }
    }
}

pub enum InitStatus {
    Created,
    AlreadyInitialized,
}

#[derive(Debug)]
pub struct Config {
    pub editor: String,
    pub memo_dir: PathBuf,
    pub memo_template: Option<PathBuf>,
    pub selector: SelectorKind,
    pub viewer: ViewerKind,
    pub grep: GrepKind,
}

#[derive(Debug)]
pub enum GrepKind {
    Builtin,
    Rg,
}

#[derive(Debug)]
pub struct ParseGrepKindError;

impl FromStr for GrepKind {
    type Err = ParseGrepKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "builtin" => Ok(GrepKind::Builtin),
            "rg" | "ripgrep" => Ok(GrepKind::Rg),
            _ => Err(ParseGrepKindError),
        }
    }
}

#[derive(Debug)]
pub enum ViewerKind {
    Builtin,
    Glow,
}

pub struct ParseViewerKindError;

impl FromStr for ViewerKind {
    type Err = ParseViewerKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "builtin" => Ok(ViewerKind::Builtin),
            "glow" => Ok(ViewerKind::Glow),
            _ => Err(ParseViewerKindError),
        }
    }
}

impl Config {
    pub fn init() -> MmemoResult<InitStatus> {
        if config_path()?.exists() {
            return Ok(InitStatus::AlreadyInitialized);
        }

        fs::create_dir_all(config_dir()?)?;
        fs::create_dir_all(mmemo_dir()?)?;
        // TODO: path渡すようにした方がいい
        Config::write_default_config()?;
        Config::write_default_template()?;

        Ok(InitStatus::Created)
    }
    fn write_default_config() -> MmemoResult<()> {
        let mut file = File::create(config_path()?)?;

        let default = concat!(
            "# Editor to use for editing memos (optional, default: vim)\n",
            "editor = \"vim\"\n",
            "\n",
            "# Directory to store memos (required)\n",
            "memo_dir = \"~/mmemo\"\n",
            "\n",
            "# Template file for new memos (optional)\n",
            "# Supports {{title}} and {{date}} } placeholders\n",
            "# Format: YAML front matter\n",
            "memo_template = \"~/.config/mmemo/template.md\"\n",
            "\n",
            "# Selector: builtin or fzf or skim (optional, default: builtin)\n",
            "selector = \"builtin\"\n",
            "\n",
            "# Viewer: builtin or glow (optional, default: builtin)\n",
            "viewer = \"builtin\"\n",
            "\n",
            "# Grep: builtin or ripgrep() (optional, default: builtin)\n",
            "grep = \"builtin\"\n"
        );

        file.write_all(default.as_bytes())?;

        Ok(())
    }
    fn write_default_template() -> MmemoResult<()> {
        let mut file = File::create(template_path()?)?;

        let default = concat!(
            "---\n",
            "title: {{title}}\n",
            "date: {{date}}\n",
            "{{tags}}\n",
            "---\n",
            "\n"
        );
        file.write_all(default.as_bytes())?;

        Ok(())
    }
    fn validate(&self) -> MmemoResult<()> {
        let memo_dir = self.memo_dir.expand_home()?;

        if !memo_dir.exists() {
            return Err(MmemoError::MemoDirNotFound(memo_dir));
        }

        if !memo_dir.is_dir() {
            return Err(MmemoError::MemoDirNotDirectory(memo_dir));
        }

        Ok(())
    }
    pub fn load() -> MmemoResult<Self> {
        let file = File::open(config_path()?).map_err(|_| MmemoError::Config {
            message: "Configuration file not found. Please run 'mmemo init'.".to_string(),
        })?;
        let tokens = Lexer::new(file)?.tokenize()?;

        let config = Config::try_from(tokens).map_err(|e| MmemoError::Config {
            message: e.0.join("\n"),
        })?;
        config.validate()?;

        Ok(config)
    }
}

#[derive(Debug)]
struct ConfigBuildError(Vec<String>);

impl TryFrom<Vec<Token>> for Config {
    type Error = ConfigBuildError;

    fn try_from(tokens: Vec<Token>) -> Result<Self, Self::Error> {
        let mut editor: Option<String> = Some("vim".into());
        let mut memo_dir: Option<PathBuf> = None;
        let mut memo_template: Option<PathBuf> = None;
        let mut selector: Option<SelectorKind> = Some(SelectorKind::Builtin);
        let mut viewer: Option<ViewerKind> = Some(ViewerKind::Builtin);
        let mut grep: Option<GrepKind> = Some(GrepKind::Builtin);

        for token in tokens {
            let value = token.value.trim();
            let value = (!value.is_empty()).then_some(value.to_string());

            match token.key {
                ConfigKey::Editor => editor = value,
                ConfigKey::MemoDir => memo_dir = value.map(PathBuf::from),
                ConfigKey::MemoTemplate => memo_template = value.map(PathBuf::from),
                ConfigKey::Selector => {
                    selector = value
                        .and_then(|s| s.parse().ok())
                        .or(Some(SelectorKind::Builtin))
                }
                ConfigKey::Viewer => {
                    viewer = value
                        .and_then(|v| v.parse().ok())
                        .or(Some(ViewerKind::Builtin))
                }
                ConfigKey::Grep => {
                    grep = value
                        .and_then(|v| v.parse().ok())
                        .or(Some(GrepKind::Builtin))
                }
            }
        }

        match (editor, memo_dir, memo_template, selector, viewer, grep) {
            (
                Some(editor),
                Some(memo_dir),
                memo_template,
                Some(selector),
                Some(viewer),
                Some(grep),
            ) => Ok(Config {
                editor,
                memo_dir,
                memo_template,
                selector,
                viewer,
                grep,
            }),
            (_, d, _, _, _, _) => {
                let vec = [(d.is_none(), "memo_dir")];

                let errors: Vec<String> = vec
                    .into_iter()
                    .filter_map(|(missing, key)| {
                        missing.then_some(format!("Please check the {} settings", key))
                    })
                    .collect();

                Err(ConfigBuildError(errors))
            }
        }
    }
}

#[derive(Debug)]
struct ParseConfigKeyError;

#[derive(Debug)]
enum ConfigKey {
    Editor,
    MemoDir,
    MemoTemplate,
    Selector,
    Viewer,
    Grep,
}

impl Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Editor => write!(f, "editor"),
            ConfigKey::MemoDir => write!(f, "memo_dir"),
            ConfigKey::MemoTemplate => write!(f, "memo_template"),
            ConfigKey::Selector => write!(f, "selector"),
            ConfigKey::Viewer => write!(f, "viewer"),
            ConfigKey::Grep => write!(f, "grep"),
        }
    }
}

impl FromStr for ConfigKey {
    type Err = ParseConfigKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "editor" => Ok(ConfigKey::Editor),
            "memo_dir" => Ok(ConfigKey::MemoDir),
            "memo_template" => Ok(ConfigKey::MemoTemplate),
            "selector" => Ok(ConfigKey::Selector),
            "viewer" => Ok(ConfigKey::Viewer),
            "grep" => Ok(ConfigKey::Grep),
            _ => Err(ParseConfigKeyError),
        }
    }
}
