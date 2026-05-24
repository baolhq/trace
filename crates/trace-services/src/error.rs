#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    InvalidInput(String),
    TitleInvalid(String),
    Db(String),
    Io(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "not found: {s}"),
            Self::InvalidInput(s) => write!(f, "invalid input: {s}"),
            Self::TitleInvalid(s) => write!(f, "title invalid: {s}"),
            Self::Db(s) => write!(f, "database error: {s}"),
            Self::Io(s) => write!(f, "io error: {s}"),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<std::io::Error> for ServiceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for ServiceError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e.to_string())
    }
}
