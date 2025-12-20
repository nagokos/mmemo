use crate::{
    Command,
    app::{config::Config, error::MmemoResult},
};

pub mod commands;
pub mod config;
pub mod error;
pub mod expand;
pub mod path_utils;

pub fn run(cmd: Command) -> MmemoResult<()> {
    match cmd {
        Command::Init => commands::init()?,
        Command::Help => todo!(),
        Command::Version => todo!(),
        _ => {
            let config = Config::load()?;
            match cmd {
                Command::New(opt) => {
                    commands::new(config, opt)?;
                }
                Command::Edit => todo!(),
                Command::Delete => todo!(),
                Command::List => todo!(),
                Command::Grep => todo!(),
                Command::Cat => todo!(),
                Command::View => todo!(),
                Command::Config => commands::config(config)?,
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}
