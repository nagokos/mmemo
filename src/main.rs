use std::{
    env::args,
    fmt::Display,
    io::{Write, stdin, stdout},
    process,
    str::FromStr,
};

mod config;
mod expand;

use crate::{config::Config, expand::HomeDir};

enum OptionsError {
    InvalidCommand(String),
    NoCommand,
}

enum ParseResult {
    Ok(Command),
    InvalidOptions(OptionsError),
    Version,
    Help,
}

#[derive(Debug)]
enum Command {
    Init,
    New(Option<Vec<String>>),
    Edit,
    Delete,
    List,
    Grep,
    Cat,
    View,
    Config,
}

impl Command {
    fn parse(args: Vec<String>) -> ParseResult {
        match args.first() {
            Some(cmd) => {
                // TODO: InvalidCommandで返せばいい
                let cmd: Command = cmd.parse().unwrap();

                match cmd {
                    Command::Init => ParseResult::Ok(Command::Init),
                    Command::New(_) => {
                        let title: Vec<String> = args.into_iter().skip(1).collect();
                        let title_opt = (!title.is_empty()).then_some(title);

                        ParseResult::Ok(Command::New(title_opt))
                    }

                    _ => todo!(),
                }
            }
            None => ParseResult::InvalidOptions(OptionsError::NoCommand),
        }
    }
    fn run(self, config: Config) {
        match self {
            Command::Init => {
                Config::init();
            }
            Command::New(opt) => match opt {
                Some(title) => {
                    let filename = format!("{}.md", title.join("_"));
                    let memo_dir = config.memo_dir.expand_home();

                    process::Command::new(config.editor)
                        .current_dir(memo_dir)
                        .arg(filename)
                        .status()
                        .unwrap();
                }
                None => {
                    let mut title = String::new();
                    print!("Title: ");
                    stdout().flush().unwrap();
                    stdin().read_line(&mut title).unwrap();

                    let filename = format!("{}.md", title.trim().replace(" ", "_"));
                    let memo_dir = config.memo_dir.expand_home();
                    process::Command::new(config.editor)
                        .current_dir(memo_dir)
                        .arg(filename)
                        .status()
                        .unwrap();
                }
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CommandParseError;

impl FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(Command::Init),
            "new" => Ok(Command::New(None)),
            "edit" => Ok(Command::Edit),
            "delete" => Ok(Command::Delete),
            "list" => Ok(Command::List),
            "grep" => Ok(Command::Grep),
            "cat" => Ok(Command::Cat),
            "view" => Ok(Command::View),
            "config" => Ok(Command::Config),
            _ => Err(CommandParseError),
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Init => write!(f, "init"),
            Command::New(s) => write!(f, "new {}", s.as_ref().unwrap().join("_")),
            Command::Edit => write!(f, "edit"),
            Command::Delete => write!(f, "delete"),
            Command::List => write!(f, "list"),
            Command::Grep => write!(f, "grep"),
            Command::Cat => write!(f, "cat"),
            Command::View => write!(f, "view"),
            Command::Config => write!(f, "config"),
        }
    }
}

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    match Command::parse(args) {
        ParseResult::Ok(cmd) => {
            let config = Config::new();
            cmd.run(config);
        }
        _ => todo!(),
    }
}
