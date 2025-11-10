use std::{ffi::OsString, path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, watch};

pub type ByteChunk = Arc<[u8]>;

#[derive(Debug, Clone)]
pub struct PtyConfig {
    pub cols: u16,
    pub rows: u16,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(OsString, OsString)>,
    pub term: Option<String>, // default: xterm-256color
}

#[derive(Debug, Clone)]
pub enum Program {
    /// Spawn the user's login shell from $SHELL (falls back to /bin/sh)
    Shell,
    /// Spawn an arbitrary argv; argv[0] must be program name/path
    Argv { argv: Vec<OsString> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sig {
    Term,
    Kill,
    Int,
    Hup,
}

#[derive(Debug)]
pub enum PtyError {
    InvalidArgs(&'static str),
    OpenFailed,
    SpawnFailed,
    Io(std::io::Error),
    ResizeFailed,
    WaitFailed,
    Unsupported,
}
impl From<std::io::Error> for PtyError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, PtyError>;

/// Receiver of output chunks (fan‑out tap).
pub type OutputRx = mpsc::Receiver<ByteChunk>;

/// Handle to a running PTY‑backed process.
pub struct PtyHandle {
    pub(crate) inner: Arc<portable::Inner>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStatus {
    pub code: u32,              // Some(code) if exited normally
    pub signal: Option<String>, // Some(name) if signaled (Unix)
}

/// Spawn a PTY process and return its handle.
pub fn spawn(program: Program, cfg: PtyConfig) -> Result<PtyHandle> {
    portable::spawn(program, cfg)
}

impl PtyHandle {
    /// Non‑blocking write to the PTY. Buffers internally.
    pub async fn write(&self, bytes: &[u8]) -> Result<usize> {
        portable::write(self, bytes).await
    }
    /// Request a resize; clamped by server‑level validation.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        portable::resize(self, cols, rows)
    }
    /// Subscribe to output. Each call creates a new tap.
    pub fn subscribe(&self) -> OutputRx {
        portable::subscribe(self)
    }
    /// Try to send a signal to the child process.
    pub fn signal(&self, sig: Sig) -> Result<()> {
        portable::signal(self, sig)
    }
    /// Watch for process exit. First Some(status) means the child has exited.
    pub fn exit_watch(&self) -> watch::Receiver<Option<ExitStatus>> {
        portable::exit_watch(self)
    }
}

mod portable;
