use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
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
    fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        for line in self.contents.lines() {
            let line: Vec<&str> = line.split("=").map(|s| s.trim()).collect();

            // TODO: 想定のキー以外であれば mmemo: 'command' is not a mmemo coomand. See 'mmemo
            // --help'
            let key: ConfigKey = line[0].parse().unwrap();
            let value = line[1].to_string();

            let token = Token { key, value };

            tokens.push(token);
        }

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
    pub memo_template: String,
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
    }
    fn write_default_config() {
        let config_path = config_path();
        if !config_path.exists() {
            let mut file = File::create(config_path).unwrap();

            let default = concat!(
                "editor = \"vim\"\n",
                "memo_dir = \"~/mmemo\"\n",
                "memo_template = \"# {title}\\n\\nDate: {date}\\n\\n\"\n"
            );

            file.write_all(default.as_bytes()).unwrap();
        }
    }
    pub fn new() -> Self {
        Config::init();

        let file = File::open(config_path()).unwrap();
        let tokens = Lexer::new(file).tokenize();

        let mut config = Config::default();
        config.apply_tokens(tokens);

        config
    }

    fn apply_tokens(&mut self, tokens: Vec<Token>) {
        for token in tokens {
            match token.key {
                ConfigKey::Editor => self.editor = token.value,
                ConfigKey::MemoDir => self.memo_dir = token.value.into(),
                ConfigKey::MemoTemplate => self.memo_template = token.value,
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

fn home_dir() -> PathBuf {
    env::home_dir().unwrap()
}

fn config_dir() -> PathBuf {
    home_dir().join(".config").join("mmemo")
}

fn mmemo_dir() -> PathBuf {
    home_dir().join("mmemo")
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}
