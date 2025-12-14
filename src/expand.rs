use std::path::{Path, PathBuf};

pub trait HomeDir {
    fn expand_home(&self) -> PathBuf {
        todo!()
    }
}

impl HomeDir for Path {
    fn expand_home(&self) -> PathBuf {
        println!("{:?}", self);
        todo!()
    }
}
