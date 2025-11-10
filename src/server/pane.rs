use crate::server::peer::PeerId;
use crate::{Error, Result};
use std::collections::{BTreeSet, HashMap};

crate::common::idgen::id_newtype!(PaneId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermSize {
    pub cols: u16,
    pub rows: u16,
}
impl TermSize {
    pub const fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }
}
impl std::fmt::Display for TermSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.cols, self.rows)
    }
}

// PTY is optional at the model level: spawn later when needed.
use crate::pty::{self, ExitStatus as PtyExit, OutputRx, Program, PtyConfig, PtyHandle, Sig as PtySig};

#[derive(Debug)]
pub enum PaneState {
    Empty,
    Running,
    Exited(PtyExit),
}

pub type SpawnTarget = Program;

pub struct Pane {
    pub id: PaneId,
    pub title: String,
    pub size: TermSize,

    pty: Option<PtyHandle>,
    taps: HashMap<PeerId, OutputRx>,
    attached: BTreeSet<PeerId>,
    input_owner: Option<PeerId>,
    state: PaneState,
}

impl std::fmt::Display for Pane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Pane id={} title={} size={}>", self.id, self.title, self.size)
    }
}

impl Pane {
    pub fn new(id: PaneId, title: impl Into<String>, size: TermSize) -> Self {
        Self {
            id,
            title: title.into(),
            size,
            pty: None,
            taps: HashMap::new(),
            attached: BTreeSet::new(),
            input_owner: None,
            state: PaneState::Empty,
        }
    }

    pub fn has_pty(&self) -> bool {
        self.pty.is_some()
    }
    pub fn is_running(&self) -> bool {
        matches!(self.state, PaneState::Running)
    }

    pub fn spawn(&mut self, target: SpawnTarget, cfg: PtyConfig) -> Result {
        if self.pty.is_some() {
            return Err(Error::InvalidState("pane already spawned".into()));
        }

        let handle = pty::spawn(target, cfg)?;
        self.pty = Some(handle);
        if let Some(ref p) = self.pty {
            for &peer in &self.attached {
                let _ = self.taps.insert(peer, p.subscribe());
            }
            if self.input_owner.is_none() {
                self.input_owner = self.attached.iter().copied().next();
            }
        }
        self.state = PaneState::Running;
        Ok(())
    }

    pub fn attach_peer(&mut self, who: PeerId) -> Result<()> {
        self.attached.insert(who);
        if let Some(ref p) = self.pty {
            let rx = p.subscribe();
            self.taps.insert(who, rx);
            if self.input_owner.is_none() {
                self.input_owner = Some(who);
            }
        }
        Ok(())
    }

    pub fn detach_peer(&mut self, who: PeerId) {
        self.attached.remove(&who);
        self.taps.remove(&who); // dropping rx closes that tap
        if self.input_owner == Some(who) {
            self.input_owner = self.attached.iter().copied().next();
        }
    }

    pub fn set_input_owner(&mut self, who: Option<PeerId>) -> Result<()> {
        if let Some(p) = who {
            if !self.attached.contains(&p) {
                return Err(Error::InvalidState("peer not attached".into()));
            }
        }
        self.input_owner = who;
        Ok(())
    }

    pub async fn write_from(&self, who: PeerId, bytes: &[u8]) -> Result<usize> {
        if self.input_owner != Some(who) {
            return Err(Error::InvalidState("peer has no input focus".into()));
        }
        let p = self.pty.as_ref().ok_or_else(|| Error::InvalidState("pane has no PTY".into()))?;
        p.write(bytes).await.map_err(|e| e.into())
    }

    pub fn resize(&mut self, size: TermSize) -> Result<()> {
        self.size = size;
        if let Some(ref p) = self.pty {
            p.resize(size.cols, size.rows)?;
        }
        Ok(())
    }

    pub fn kill(&mut self, force: bool) -> Result<()> {
        if let Some(ref p) = self.pty {
            let sig = if force { PtySig::Kill } else { PtySig::Term };
            p.signal(sig)?;
        }
        Ok(())
    }

    /// Non-blocking state refresh from PTY exit_watch.
    pub fn poll_exit(&mut self) {
        if let Some(ref p) = self.pty {
            let rx = p.exit_watch();
            if let Some(ref status) = *rx.borrow() {
                self.state = PaneState::Exited(status.to_owned());
            }
        }
    }

    pub fn tap(&mut self, peer: PeerId) -> Option<&mut OutputRx> {
        self.taps.get_mut(&peer)
    }

    pub fn take_tap(&mut self, peer: PeerId) -> Option<OutputRx> {
        self.taps.remove(&peer)
    }

    pub fn state(&self) -> &PaneState {
        &self.state
    }
}
