use std::{env::args, fmt::Display, str::FromStr};

use crate::app::run;

mod app;

#[derive(Debug, thiserror::Error)]
enum CliParseError {
    // TODO: cr helloとかでParse error: helloになるのはおかしいので考える
    // helpと同じような感じでもいいのかも
    #[error("Parse error: {command}")]
    UnknownCommand { command: String },

    #[error("command not found")]
    CommandNotFound,

    #[error("Usage: {usage}")]
    MissingArgument { usage: String },
}

#[derive(Debug)]
enum Command {
    Init,
    New(Vec<String>),
    Edit,
    Delete,
    List,
    Grep(Vec<String>, String),
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
                        if title.is_empty() {
                            return Err(CliParseError::MissingArgument {
                                usage: "mmemo new <title>".to_string(),
                            });
                        }
                        Ok(Command::New(title))
                    }
                    Command::Edit => Ok(Command::Edit),
                    Command::Delete => Ok(Command::Delete),
                    Command::List => Ok(Command::List),
                    Command::View => Ok(Command::View),
                    Command::Grep(_, _) => {
                        let mut options = Vec::new();
                        let mut target = String::new();

                        for arg in args.into_iter().skip(1) {
                            if arg.starts_with("-") {
                                options.push(arg);
                                continue;
                            } else {
                                target = arg;
                            }
                        }

                        if target.is_empty() {
                            return Err(CliParseError::MissingArgument {
                                usage: "mmemo grep <target>".to_string(),
                            });
                        }
                        Ok(Command::Grep(options, target))
                    }
                    Command::Config => Ok(Command::Config),
                    Command::Help => todo!(),
                    Command::Version => todo!(),
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
            "init" | "i" => Ok(Command::Init),
            "new" | "n" => Ok(Command::New(Vec::new())),
            "edit" | "e" => Ok(Command::Edit),
            "delete" | "d" => Ok(Command::Delete),
            "list" | "l" => Ok(Command::List),
            "grep" | "g" => Ok(Command::Grep(Vec::new(), String::new())),
            "view" | "v" => Ok(Command::View),
            "config" | "c" => Ok(Command::Config),
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
            Command::New(s) => write!(f, "new {}", s.join(" ")),
            Command::Edit => write!(f, "edit"),
            Command::Delete => write!(f, "delete"),
            Command::List => write!(f, "list"),
            Command::Grep(o, s) => write!(f, "grep {} {}", o.join(" "), s),
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
