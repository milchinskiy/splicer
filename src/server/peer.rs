crate::common::idgen::id_newtype!(PeerId);

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: PeerId,
    pub name: String,
    // future: perms, caps, palette, etc.
}

impl Peer {
    pub fn new(id: PeerId, name: impl Into<String>) -> Self {
        Self { id, name: name.into() }
    }
}

impl std::fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Peer id={} name={}>", self.id, self.name)
    }
}
