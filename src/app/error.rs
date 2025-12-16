#[derive(Debug, thiserror::Error)]
pub enum MmemoError {
    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("File operation failed: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Memo search not found: {query}")]
    MemoNotFound { query: String },
}

pub type Result<T> = std::result::Result<T, MmemoError>;
