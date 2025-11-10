use super::pane::{Pane, PaneId};
use crate::{Error, Result};
use std::collections::BTreeMap;

crate::common::idgen::id_newtype!(WindowId);

pub struct Window {
    pub id: WindowId,
    pub name: String,
    panes: BTreeMap<PaneId, Pane>,
    focused: Option<PaneId>,
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Window id={} name={} focused={:?} panes={}>", self.id, self.name, self.focused, self.panes.len())
    }
}

impl Window {
    pub fn new(id: WindowId, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), panes: BTreeMap::new(), focused: None }
    }

    pub fn add_pane(&mut self, pane: Pane) -> Result<()> {
        if self.panes.contains_key(&pane.id) {
            return Err(Error::InvalidState("pane already exists".into()));
        }
        let id = pane.id;
        self.panes.insert(id, pane);
        if self.focused.is_none() {
            self.focused = Some(id);
        }
        Ok(())
    }

    pub fn remove_pane(&mut self, id: PaneId) -> Option<Pane> {
        let removed = self.panes.remove(&id);
        if self.focused == Some(id) {
            self.focused = self.panes.keys().next().copied();
        }
        removed
    }

    pub fn focus(&mut self, id: PaneId) -> Result<()> {
        if !self.panes.contains_key(&id) {
            return Err(Error::InvalidState("pane not found".into()));
        }
        self.focused = Some(id);
        Ok(())
    }

    pub fn focused(&self) -> Option<PaneId> {
        self.focused
    }
    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.get(&id)
    }
    pub fn pane_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        self.panes.get_mut(&id)
    }
    pub fn panes(&self) -> impl Iterator<Item = (&PaneId, &Pane)> {
        self.panes.iter()
    }
}
