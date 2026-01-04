use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum MmemoError {
    #[error("Environment variable not set: {key}")]
    EnvVarMissing { key: &'static str },

    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("File operation failed: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Memo directory not found: {0}")]
    MemoDirNotFound(PathBuf),

    #[error("Not a directory: {0}")]
    MemoDirNotDirectory(PathBuf),

    #[error("Memo search not found: {query}")]
    MemoNotFound { query: String },
}

pub type MmemoResult<T> = std::result::Result<T, MmemoError>;
