#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Lua(mlua::Error),
    Ipc(String),
    Pty(String),
    InvalidState(String),
    Timeout(String),
    UserInput(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Lua(e) => write!(f, "Lua error: {e}"),
            Self::UserInput(e) => write!(f, "User input error: {e}"),
            Self::Ipc(e) => write!(f, "IPC error: {e}"),
            Self::Pty(e) => write!(f, "Pty error: {e}"),
            Self::InvalidState(e) => write!(f, "Invalid state: {e}"),
            Self::Timeout(e) => write!(f, "Timeout: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<mlua::Error> for Error {
    fn from(e: mlua::Error) -> Self {
        Self::Lua(e)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(e: std::num::TryFromIntError) -> Self {
        Self::InvalidState(e.to_string())
    }
}
