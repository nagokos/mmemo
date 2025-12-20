use std::{env::args, fmt::Display, str::FromStr};

use crate::app::run;

mod app;

#[derive(Debug, thiserror::Error)]
enum CliParseError {
    #[error("Parse error: {command}")]
    UnknownCommand { command: String },

    #[error("command not found")]
    CommandNotFound,
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
    Help,
    Version,
}

impl Command {
    fn parse(args: Vec<String>) -> Result<Command, CliParseError> {
        match args.first() {
            Some(cmd) => {
                let cmd: Command = cmd.parse().map_err(|_| CliParseError::UnknownCommand {
                    command: cmd.to_string(),
                })?;

                match cmd {
                    Command::Init => Ok(Command::Init),
                    Command::New(_) => {
                        let title: Vec<String> = args.into_iter().skip(1).collect();
                        let title_opt = (!title.is_empty()).then_some(title);

                        Ok(Command::New(title_opt))
                    }
                    Command::Config => Ok(Command::Config),
                    _ => todo!(),
                }
            }
            None => Err(CliParseError::CommandNotFound),
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
            "help" | "-h" | "--help" => Ok(Command::Help),
            "version" | "-v" | "--version" => Ok(Command::Version),
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
            Command::Help => write!(f, "help"),
            Command::Version => write!(f, "version"),
        }
    }
}

fn main() {
    let args: Vec<String> = args().skip(1).collect();

    let cmd = Command::parse(args).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    if let Err(e) = run(cmd) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
