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
        Command::Help => todo!(),
        Command::Version => todo!(),
        Command::New(s) => commands::new(Config::load()?, s)?,
        Command::Edit => commands::edit(Config::load()?)?,
        Command::Delete => commands::delete(Config::load()?)?,
        Command::List => commands::list(Config::load()?)?,
        Command::Grep(o, s) => commands::grep(Config::load()?, o, s)?,
        Command::View => commands::view(Config::load()?)?,
        Command::Config => commands::config(Config::load()?)?,
    }
    Ok(())
}
