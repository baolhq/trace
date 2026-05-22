#[derive(Debug)]
pub enum CoreError {
    NotFound(String),
    InvalidId(String),
    Parse(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "not found: {s}"),
            Self::InvalidId(s) => write!(f, "invalid id: {s}"),
            Self::Parse(s) => write!(f, "parse error: {s}"),
        }
    }
}

impl std::error::Error for CoreError {}
