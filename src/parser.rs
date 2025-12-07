use std::{
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
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
        for line in self.contents.lines() {
            let line: Vec<&str> = line.split("=").collect();

            let config_key: ConfigKey = line[0].parse().unwrap();
            println!("{}", config_key);
        }

        todo!()
    }
}

struct Token {
    key: ConfigKey,
    value: String,
}

#[derive(Debug)]
pub struct Config {
    editor: String,
    memo_dir: PathBuf,
    memo_template: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: "vim".to_string(),
            memo_dir: PathBuf::from(std::env::var("HOME").unwrap() + "/mmemo"),
            memo_template: "# {title}\n\nDate: {date}\n\n".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        // TODO: 開発環境では./config.tomlにしておく
        // let path = std::env::var("HOME").unwrap() + "/mmemo" + "/config.toml";
        let path = "config.toml";
        let path = Path::new(&path);

        let file = File::open(path).unwrap();
        let token = Lexer::new(file).tokenize();

        Config::default()
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {}
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
