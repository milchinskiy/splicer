use super::{
    pane::{Pane, PaneId, TermSize},
    peer::{Peer, PeerId},
    session::{Session, SessionId},
    window::{Window, WindowId},
};
use crate::server::IdAllocator;
use crate::{Error, Result};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Allocators {
    session: IdAllocator,
    window: IdAllocator,
    pane: IdAllocator,
    peer: IdAllocator,
}

#[derive(Default)]
pub struct ServerState {
    allocs: Allocators,
    sessions: BTreeMap<SessionId, Session>,
    peers: BTreeMap<PeerId, Peer>,
}

impl std::fmt::Display for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ServerState sessions={} peers={}>", self.sessions.len(), self.peers.len())
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_session(&mut self, name: impl Into<String>) -> SessionId {
        let id = self.allocs.session.allocate(SessionId::new);
        self.sessions.insert(id, Session::new(id, name));
        id
    }

    pub fn add_window(&mut self, sid: SessionId, win: Window) -> Result<()> {
        let s = self.sessions.get_mut(&sid).ok_or_else(|| Error::InvalidState("no such session".into()))?;
        s.add_window(win)
    }

    pub fn new_window(&mut self, sid: SessionId, name: impl Into<String>) -> Result<WindowId> {
        let id = self.allocs.window.allocate(WindowId::new);
        let w = Window::new(id, name);
        self.add_window(sid, w)?;
        Ok(id)
    }

    pub fn new_peer(&mut self, name: impl Into<String>) -> PeerId {
        let id = self.allocs.peer.allocate(PeerId::new);
        self.peers.insert(id, Peer::new(id, name));
        id
    }

    pub fn add_pane(&mut self, sid: SessionId, wid: WindowId, pane: Pane) -> Result<()> {
        let s = self.sessions.get_mut(&sid).ok_or_else(|| Error::InvalidState("no such session".into()))?;
        let w = s.window_mut(wid).ok_or_else(|| Error::InvalidState("no such window".into()))?;
        w.add_pane(pane)
    }

    pub fn new_pane(
        &mut self,
        sid: SessionId,
        wid: WindowId,
        title: impl Into<String>,
        size: TermSize,
    ) -> Result<PaneId> {
        let id = self.allocs.pane.allocate(PaneId::new);
        let pane = Pane::new(id, title, size);
        self.add_pane(sid, wid, pane)?;
        Ok(id)
    }

    pub fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }
    pub fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(&id)
    }
    pub fn sessions(&self) -> impl Iterator<Item = (&SessionId, &Session)> {
        self.sessions.iter()
    }

    pub fn peer(&self, id: PeerId) -> Option<&Peer> {
        self.peers.get(&id)
    }
    pub fn peers(&self) -> impl Iterator<Item = (&PeerId, &Peer)> {
        self.peers.iter()
    }
}
