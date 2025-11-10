use super::peer::PeerId;
use super::window::{Window, WindowId};
use crate::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};

crate::common::idgen::id_newtype!(SessionId);

pub struct Session {
    pub id: SessionId,
    pub name: String,
    windows: BTreeMap<WindowId, Window>,
    focused: Option<WindowId>,
    peers: BTreeSet<PeerId>,
}

impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Session id={} name={} windows={} peers={} focused={:?}>",
            self.id,
            self.name,
            self.windows.len(),
            self.peers.len(),
            self.focused
        )
    }
}

impl Session {
    pub fn new(id: SessionId, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), windows: BTreeMap::new(), focused: None, peers: BTreeSet::new() }
    }

    pub fn add_window(&mut self, win: Window) -> Result<()> {
        if self.windows.contains_key(&win.id) {
            return Err(Error::InvalidState("window already exists".into()));
        }
        let id = win.id;
        self.windows.insert(id, win);
        if self.focused.is_none() {
            self.focused = Some(id);
        }
        Ok(())
    }

    pub fn remove_window(&mut self, id: WindowId) -> Option<Window> {
        let removed = self.windows.remove(&id);
        if self.focused == Some(id) {
            self.focused = self.windows.keys().next().copied();
        }
        removed
    }

    pub fn focus_window(&mut self, id: WindowId) -> Result<()> {
        if !self.windows.contains_key(&id) {
            return Err(Error::InvalidState("window not found".into()));
        }
        self.focused = Some(id);
        Ok(())
    }

    pub fn attach_peer(&mut self, who: PeerId) {
        self.peers.insert(who);
    }
    pub fn detach_peer(&mut self, who: PeerId) {
        self.peers.remove(&who);
    }

    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }
    pub fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }
    pub fn focused(&self) -> Option<WindowId> {
        self.focused
    }
    pub fn windows(&self) -> impl Iterator<Item = (&WindowId, &Window)> {
        self.windows.iter()
    }
}
