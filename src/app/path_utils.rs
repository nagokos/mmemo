use std::{env, path::PathBuf};

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME environment variable not set")
}
