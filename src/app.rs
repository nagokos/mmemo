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
                    commands::new()?;
                }
                Command::Edit => todo!(),
                Command::Delete => todo!(),
                Command::List => todo!(),
                Command::Grep => todo!(),
                Command::Cat => todo!(),
                Command::View => todo!(),
                Command::Config => todo!(),
                _ => unreachable!(),
            }
        }
    }

    // fn run(self, config: Config) {
    //     match self {
    //         Command::Init => {
    //             Config::init();
    //         }
    //         Command::New(opt) => match opt {
    //             Some(title) => {
    //                 let filename = format!("{}.md", title.join("_"));
    //                 let memo_dir = config.memo_dir.expand_home();
    //
    //                 process::Command::new(config.editor)
    //                     .current_dir(memo_dir)
    //                     .arg(filename)
    //                     .status()
    //                     .unwrap();
    //             }
    //             None => {
    //                 let mut title = String::new();
    //                 print!("Title: ");
    //                 stdout().flush().unwrap();
    //                 stdin().read_line(&mut title).unwrap();
    //
    //                 let filename = format!("{}.md", title.trim().replace(" ", "_"));
    //                 let memo_dir = config.memo_dir.expand_home();
    //                 process::Command::new(config.editor)
    //                     .current_dir(memo_dir)
    //                     .arg(filename)
    //                     .status()
    //                     .unwrap();
    //             }
    //         },
    //         _ => todo!(),
    //     }
    // }

    Ok(())
}
