use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::app::selector::Selector;

pub struct Fzf;
impl Selector for Fzf {
    fn select(&self, items: Vec<String>) -> std::io::Result<Option<String>> {
        run("fzf", items)
    }
}

pub struct Skim;
impl Selector for Skim {
    fn select(&self, items: Vec<String>) -> std::io::Result<Option<String>> {
        run("sk", items)
    }
}

fn run(command: &str, items: Vec<String>) -> std::io::Result<Option<String>> {
    let mut selector = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = selector.stdin.as_mut().unwrap();
    for item in items {
        writeln!(&mut stdin, "{}", item)?;
    }

    let output = selector.wait_with_output()?;
    let select = (!output.stdout.is_empty())
        .then_some(String::from_utf8_lossy(output.stdout.trim_ascii()).to_string());

    Ok(select)
}
