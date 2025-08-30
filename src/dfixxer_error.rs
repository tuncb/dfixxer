use std::fmt;

#[derive(Debug)]
pub enum DFixxerError {
    InvalidArgs(String),
    IoError(std::io::Error),
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for DFixxerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DFixxerError::InvalidArgs(msg) => write!(f, "{}", msg),
            DFixxerError::IoError(err) => write!(f, "Failed to read file: {}", err),
            DFixxerError::ParseError(msg) => write!(f, "{}", msg),
            DFixxerError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for DFixxerError {}

impl From<std::io::Error> for DFixxerError {
    fn from(err: std::io::Error) -> Self {
        DFixxerError::IoError(err)
    }
}
