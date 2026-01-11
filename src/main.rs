use std::env::args;

use crate::app::run;

mod app;

#[derive(Debug, thiserror::Error)]
enum CliParseError {
    // TODO: cr helloとかでParse error: helloになるのはおかしいので考える
    // helpと同じような感じでもいいのかも
    #[error("unknown command: mmemo {command}")]
    UnknownCommand { command: String },

    #[error("usage: {usage}")]
    MissingArgument { usage: String },
}

#[derive(Debug)]
enum Command {
    Init,
    New(String),
    Edit,
    Delete,
    List,
    Grep(Vec<String>),
    View,
    Config,
    Help,
    Version,
}

impl TryFrom<Vec<String>> for Command {
    type Error = CliParseError;

    fn try_from(args: Vec<String>) -> Result<Self, Self::Error> {
        match args.first() {
            Some(s) => match s.as_str() {
                "init" | "i" => Ok(Command::Init),
                "new" | "n" => {
                    let title: Vec<String> = args.into_iter().skip(1).collect();
                    if title.is_empty() {
                        return Err(CliParseError::MissingArgument {
                            usage: "mmemo new <title>".to_string(),
                        });
                    }
                    Ok(Command::New(title.join(" ")))
                }
                "edit" | "e" => Ok(Command::Edit),
                "delete" | "d" => Ok(Command::Delete),
                "list" | "l" => Ok(Command::List),
                "grep" | "g" => {
                    let rest: Vec<String> = args.into_iter().skip(1).collect();
                    if rest.is_empty() {
                        return Err(CliParseError::MissingArgument {
                            usage: "mmemo grep <pattern...>".to_string(),
                        });
                    }
                    Ok(Command::Grep(rest))
                }
                "view" | "v" => Ok(Command::View),
                "config" | "c" => Ok(Command::Config),
                // TODO: commandとして扱わないでここでやるとか
                "-h" | "--help" => Ok(Command::Help),
                "-v" | "--version" => Ok(Command::Version),
                _ => Err(CliParseError::UnknownCommand { command: s.into() }),
            },
            None => Ok(Command::Help),
        }
    }
}

fn main() {
    let args: Vec<String> = args().skip(1).collect();

    let cmd: Command = args.try_into().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    if let Err(e) = run(cmd) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
