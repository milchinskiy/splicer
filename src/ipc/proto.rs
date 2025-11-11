use crate::server::{pane::PaneId, peer::PeerId, session::SessionId, window::WindowId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ErrorCode {
    Ok = 0,
    NotFound = 2,
    InvalidArgs = 3,
    NotAttached = 4,
    VersionMismatch = 5,
    Denied = 6,
    Timeout = 7,
    Internal = 255,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hello {
    pub client_api_major: u8,
    pub features: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct HelloAck {
    pub server_api_major: u8,
    pub features: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    CreateSession {
        name: Option<String>,
    },
    ListSessions,
    CreateWindow {
        session: SessionId,
        title: Option<String>,
    },
    SpawnPane {
        session: SessionId,
        window: Option<WindowId>,
        title: Option<String>,
        cwd: Option<String>,
        argv: Vec<String>,
    },
    Attach {
        session: SessionId,
        window: Option<WindowId>,
        pane: Option<PaneId>,
    },
    Detach {
        target: Option<DetachTarget>,
    },
    Kill {
        target: KillTarget,
        force: bool,
    },
    GetState {
        scope: StateScope,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ok,
    SessionCreated { session: SessionId },
    Sessions { items: Vec<SessionLite> },
    WindowCreated { window: WindowId },
    PaneSpawned { session: SessionId, window: WindowId, pane: PaneId },
    Attached,
    Detached,
    Killed,
    State { json: serde_json::Value },
    Err { code: ErrorCode, msg: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DetachTarget {
    Session(SessionId),
    Window(WindowId),
    Pane(PaneId),
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum KillTarget {
    Session(SessionId),
    Window(WindowId),
    Pane(PaneId),
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum StateScope {
    Sessions,
    Windows { session: Option<SessionId> },
    Panes { window: Option<WindowId> },
    Peers,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionLite {
    pub id: SessionId,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    // Use Vec<u8> for serde compatibility; convert from your ByteChunk on send
    PtyOutput { pane: PaneId, chunk: Vec<u8> },
    TitleChanged { window: WindowId, title: String },
    LayoutChanged { window: WindowId },
    PeerAttached { peer: PeerId, session: SessionId, window: WindowId, pane: PaneId },
    PeerDetached { peer: PeerId },
    Bye { reason: String },
    StreamDropNotice { pane: PaneId },
}
