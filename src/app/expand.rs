use std::path::{Path, PathBuf};

use crate::app::{error::MmemoResult, path_utils};

pub trait HomeDir {
    fn expand_home(&self) -> MmemoResult<PathBuf>;
}

impl HomeDir for Path {
    fn expand_home(&self) -> MmemoResult<PathBuf> {
        match self.to_str() {
            Some(s) => {
                if s == "~" {
                    Ok(path_utils::home_dir()?)
                } else {
                    let path = s
                        .strip_prefix("~/")
                        .and_then(|rest| path_utils::home_dir().ok().map(|path| path.join(rest)))
                        .unwrap_or_else(|| self.to_path_buf());
                    Ok(path)
                }
            }
            None => Ok(self.to_path_buf()),
        }
    }
}
