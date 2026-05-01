use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Svn(String),
    Ai(String),
    Fs(String),
    Config(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Svn(msg) => write!(f, "[SVN] {}", msg),
            AppError::Ai(msg) => write!(f, "[AI] {}", msg),
            AppError::Fs(msg) => write!(f, "[FS] {}", msg),
            AppError::Config(msg) => write!(f, "[CFG] {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}
