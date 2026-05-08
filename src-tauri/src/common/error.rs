use serde::Serialize;

/// SVN 错误类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "detail")]
pub enum SvnError {
    /// 命令执行超时
    Timeout { seconds: u64 },
    /// 进程启动失败
    Spawn(String),
    /// 命令返回非零退出码
    ExitCode { code: i32, stderr: String },
    /// IO 错误
    Io(String),
    /// 输出解析失败
    Parse(String),
    /// SVN 可执行文件未找到
    ExecutableNotFound,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for SvnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SvnError::Timeout { seconds } => write!(f, "SVN command timed out after {} seconds", seconds),
            SvnError::Spawn(msg) => write!(f, "Failed to spawn SVN process: {}", msg),
            SvnError::ExitCode { code, stderr } => {
                if stderr.is_empty() {
                    write!(f, "SVN command failed with exit code {}", code)
                } else {
                    write!(f, "SVN command failed: {}", stderr.trim())
                }
            }
            SvnError::Io(msg) => write!(f, "SVN IO error: {}", msg),
            SvnError::Parse(msg) => write!(f, "Failed to parse SVN output: {}", msg),
            SvnError::ExecutableNotFound => write!(f, "SVN command line tool not found. Please install SVN client or configure path."),
            SvnError::Other(msg) => write!(f, "SVN error: {}", msg),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    Svn(SvnError),
    Ai(String),
    Fs(String),
    Config(String),
}

impl AppError {
    /// 创建 SVN 超时错误
    pub fn svn_timeout(seconds: u64) -> Self {
        AppError::Svn(SvnError::Timeout { seconds })
    }

    /// 创建 SVN spawn 错误
    pub fn svn_spawn(msg: impl Into<String>) -> Self {
        AppError::Svn(SvnError::Spawn(msg.into()))
    }

    /// 创建 SVN 退出码错误
    pub fn svn_exit_code(code: i32, stderr: impl Into<String>) -> Self {
        AppError::Svn(SvnError::ExitCode { code, stderr: stderr.into() })
    }

    /// 创建 SVN IO 错误
    pub fn svn_io(msg: impl Into<String>) -> Self {
        AppError::Svn(SvnError::Io(msg.into()))
    }

    /// 创建 SVN 解析错误
    pub fn svn_parse(msg: impl Into<String>) -> Self {
        AppError::Svn(SvnError::Parse(msg.into()))
    }

    /// 创建 SVN 可执行文件未找到错误
    pub fn svn_not_found() -> Self {
        AppError::Svn(SvnError::ExecutableNotFound)
    }

    /// 创建 SVN 其他错误
    pub fn svn_other(msg: impl Into<String>) -> Self {
        AppError::Svn(SvnError::Other(msg.into()))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Svn(e) => write!(f, "[SVN] {}", e),
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

impl From<SvnError> for AppError {
    fn from(e: SvnError) -> Self {
        AppError::Svn(e)
    }
}
