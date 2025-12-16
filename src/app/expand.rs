use std::path::{Path, PathBuf};

use crate::app::path_utils;

pub trait HomeDir {
    fn expand_home(&self) -> PathBuf {
        todo!()
    }
}

impl HomeDir for Path {
    fn expand_home(&self) -> PathBuf {
        match self.to_str() {
            Some(s) => {
                if s == "~" {
                    path_utils::home_dir()
                } else {
                    s.strip_prefix("~/")
                        .map(|rest| path_utils::home_dir().join(rest))
                        .unwrap_or_else(|| self.to_path_buf())
                }
            }
            None => self.to_path_buf(),
        }
    }
}
