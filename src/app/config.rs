use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    process::{self},
    str::FromStr,
};

use toml::Table;

use crate::path_utils;

struct Lexer {
    contents: String,
}

impl Lexer {
    fn new(mut file: File) -> Self {
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        Self { contents: buf }
    }
    fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        // let table = line.parse::<Table>().unwrap();
        //
        // for (key, val) in &table {
        //     let key  = match key.parse::<ConfigKey>() {
        //         Ok(key) =>
        //     };
        //     let value = val.to_string();
        //
        //     let token = Token { key, value };
        //     tokens.push(token);
        // }

        tokens
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

impl Config {
    pub fn init() {
        if config_path().exists() {
            print!(
                "It has already been initialized. \nThe configuration file exists at ~/.config/mmemo/config.toml",
            );
            process::exit(0)
        }

        fs::create_dir_all(config_dir()).unwrap();
        fs::create_dir_all(mmemo_dir()).unwrap();
        Config::write_default_config();
        Config::write_default_template();
    }
    fn write_default_config() {
        let config_path = config_path();
        let mut file = File::create(config_path).unwrap();

        let default = concat!(
            "editor = \"vim\"\n",
            "memo_dir = \"~/mmemo\"\n",
            // これもパスにしてしまう
            "memo_template = \"~/.config/mmemo/template.md\"\n"
        );

        file.write_all(default.as_bytes()).unwrap();
    }
    fn write_default_template() {
        let template_path = template_path();
        let mut file = File::create(template_path).unwrap();

        let default = "# {title}\n\nDate: {date}\n\n";
        file.write_all(default.as_bytes()).unwrap();
    }
    pub fn new() -> Self {
        let file = File::open(config_path()).unwrap();
        let tokens = Lexer::new(file).tokenize();

        Config::try_from(tokens).unwrap_or_else(|e| {
            e.0.into_iter().for_each(|msg| println!("{msg}"));
            std::process::exit(1);
        })
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

fn config_dir() -> PathBuf {
    path_utils::home_dir().join(".config").join("mmemo")
}

fn mmemo_dir() -> PathBuf {
    path_utils::home_dir().join("mmemo")
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

fn template_path() -> PathBuf {
    config_dir().join("template.md")
}
