use crate::{
    Command,
    app::{config::Config, error::MmemoResult},
};

pub mod commands;
pub mod config;
pub mod error;
pub mod expand;
pub mod path_utils;

pub mod selector;
pub mod template;

pub fn run(cmd: Command) -> MmemoResult<()> {
    match cmd {
        Command::Init => commands::init()?,
        Command::Help => commands::help(),
        Command::Version => commands::version(),
        _ => {
            let config = Config::load()?;
            match cmd {
                Command::New(s) => commands::new(&config, &s)?,
                Command::Edit => commands::edit(&config)?,
                Command::Delete => commands::delete(&config)?,
                Command::List => commands::list(&config)?,
                Command::Grep(o, s) => commands::grep(&config, &o, &s)?,
                Command::View => commands::view(&config)?,
                Command::Config => commands::config(&config)?,
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}
