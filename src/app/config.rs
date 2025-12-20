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
    path_utils::{config_dir, config_path, mmemo_dir, template_path},
};

struct Lexer {
    contents: String,
}

impl Lexer {
    fn new(mut file: File) -> Self {
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        Self { contents: buf }
    }
    fn tokenize(&self) -> MmemoResult<Vec<Token>> {
        let mut tokens = Vec::new();

        let table = toml::from_str::<Table>(&self.contents).unwrap();

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
pub struct Config {
    pub editor: String,
    pub memo_dir: PathBuf,
    pub memo_template: Option<PathBuf>,
}

pub enum InitStatus {
    Created,
    AlreadyInitialized,
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
            "editor = \"vim\"\n",
            "memo_dir = \"~/mmemo\"\n",
            // これもパスにしてしまう
            "memo_template = \"~/.config/mmemo/template.md\"\n"
        );

        file.write_all(default.as_bytes())?;

        Ok(())
    }
    fn write_default_template() -> MmemoResult<()> {
        let mut file = File::create(template_path()?)?;

        let default = "# {title}\n\nDate: {date}\n\n";
        file.write_all(default.as_bytes())?;

        Ok(())
    }
    pub fn load() -> MmemoResult<Self> {
        let file = File::open(config_path()?).map_err(|_| MmemoError::Config {
            message: "Configuration file not found. Please run 'mmemo init'.".to_string(),
        })?;
        let tokens = Lexer::new(file).tokenize()?;

        let config = Config::try_from(tokens).map_err(|e| MmemoError::Config {
            message: e.0.join("\n"),
        })?;

        Ok(config)
    }
}

#[derive(Debug)]
struct ConfigBuildError(Vec<String>);

impl TryFrom<Vec<Token>> for Config {
    type Error = ConfigBuildError;

    fn try_from(tokens: Vec<Token>) -> Result<Self, Self::Error> {
        let mut editor: Option<String> = None;
        let mut memo_dir: Option<String> = None;
        let mut memo_template: Option<String> = None;

        for token in tokens {
            let value = token.value.trim();
            let value = (!value.is_empty()).then_some(value.to_string());

            match token.key {
                ConfigKey::Editor => editor = value,
                ConfigKey::MemoDir => memo_dir = value,
                ConfigKey::MemoTemplate => memo_template = value,
            }
        }

        match (editor, memo_dir, memo_template) {
            (Some(editor), Some(memo_dir), memo_template) => Ok(Config {
                editor,
                memo_dir: memo_dir.into(),
                memo_template: memo_template.map(|t| t.into()),
            }),
            (e, d, _) => {
                let vec = [(e.is_none(), "editor"), (d.is_none(), "memo_dir")];

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

#[derive(Debug, PartialEq, Eq)]
struct ParseConfigKeyError;

#[derive(Debug)]
enum ConfigKey {
    Editor,
    MemoDir,
    MemoTemplate,
}

impl Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Editor => write!(f, "editor"),
            ConfigKey::MemoDir => write!(f, "memo_dir"),
            ConfigKey::MemoTemplate => write!(f, "memo_template"),
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
            _ => Err(ParseConfigKeyError),
        }
    }
}
