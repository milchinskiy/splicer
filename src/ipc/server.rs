use super::proto::*;
use super::wire::*;
use crate::server::peer::PeerId;
use crate::{Error, Result};
use rmp_serde::{from_slice, to_vec};
use tokio::{
    net::{UnixListener, UnixStream},
    sync::{mpsc, oneshot},
};

pub struct IpcServer {
    lis: UnixListener,
    core_tx: mpsc::Sender<CoreMsg>,
}

pub enum CoreMsg {
    RegisterPeer { ev_tx: mpsc::Sender<Event>, reply: oneshot::Sender<PeerId> },
    FromPeer { peer: PeerId, req: Request, reply: oneshot::Sender<Response> },
    UnregisterPeer { peer: PeerId },
}

impl IpcServer {
    pub async fn bind(path: &str, core_tx: mpsc::Sender<CoreMsg>) -> Result<Self> {
        // tip: unlink stale socket and set 0700 elsewhere
        Ok(Self { lis: UnixListener::bind(path)?, core_tx })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let (sock, _addr) = self.lis.accept().await?;
            let tx = self.core_tx.clone();
            tokio::spawn(async move {
                let _ = handle_peer(sock, tx).await;
            });
        }
    }
}

struct Outbound {
    hdr: FrameHeader,
    bytes: Vec<u8>,
}

async fn handle_peer(sock: UnixStream, core_tx: mpsc::Sender<CoreMsg>) -> Result<()> {
    let (mut r, w) = sock.into_split();

    // Dedicated writer task owns the write half
    let (write_tx, mut write_rx) = mpsc::channel::<Outbound>(256);
    let writer = tokio::spawn(async move {
        let mut w = w;
        while let Some(Outbound { hdr, bytes }) = write_rx.recv().await {
            write_payload(&mut w, hdr, &bytes).await?;
        }
        Ok::<(), std::io::Error>(())
    });

    // Handshake (read first frame, reply via writer)
    let (hdr0, bytes0) = read_payload(&mut r).await?;
    if hdr0.schema_id != 0 {
        return Err(Error::InvalidState("expected Hello".into()));
    }
    let hello: Hello = from_slice(&bytes0).map_err(|e| Error::Ipc(e.to_string()))?;

    let ack = HelloAck { server_api_major: 1, features: 0 };
    let ack_bytes = to_vec(&ack).unwrap();
    let ack_hdr = FrameHeader { api_major: 1, kind: Kind::Response, schema_id: 0, len: ack_bytes.len() as u32 };
    let _ = write_tx.send(Outbound { hdr: ack_hdr, bytes: ack_bytes }).await;
    if hello.client_api_major != 1 {
        return Ok(());
    }

    // Register peer with Core — Core allocates PeerId
    let (ev_tx, mut ev_rx) = mpsc::channel::<Event>(256);
    let (id_tx, id_rx) = oneshot::channel();
    core_tx.send(CoreMsg::RegisterPeer { ev_tx: ev_tx.clone(), reply: id_tx }).await.ok();
    let peer = id_rx.await.map_err(|_| Error::InvalidState("peer id alloc failed".into()))?;

    // Event forwarder → writer
    let write_tx_events = write_tx.clone();
    let ev_forward = tokio::spawn(async move {
        while let Some(ev) = ev_rx.recv().await {
            let bytes = rmp_serde::to_vec(&ev).unwrap();
            let hdr = FrameHeader { api_major: 1, kind: Kind::Event, schema_id: 2, len: bytes.len() as u32 };
            if write_tx_events.send(Outbound { hdr, bytes }).await.is_err() {
                break;
            }
        }
        Ok::<(), ()>(())
    });

    // Reader: route requests → core; core response → writer
    let tx_core = core_tx.clone();
    let write_tx_resp = write_tx.clone();
    let reader = tokio::spawn(async move {
        loop {
            let (hdr, bytes) =
                read_payload(&mut r).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))?;
            if hdr.schema_id != 1 || !matches!(hdr.kind, Kind::Request) {
                continue;
            }
            let req: Request =
                rmp_serde::from_slice(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let (reply_tx, reply_rx) = oneshot::channel();
            tx_core
                .send(CoreMsg::FromPeer { peer, req, reply: reply_tx })
                .await
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "core gone"))?;
            if let Ok(resp) = reply_rx.await {
                let bytes = rmp_serde::to_vec(&resp).unwrap();
                let hdr = FrameHeader { api_major: 1, kind: Kind::Response, schema_id: 1, len: bytes.len() as u32 };
                if write_tx_resp.send(Outbound { hdr, bytes }).await.is_err() {
                    break;
                }
            }
        }
        Ok::<(), std::io::Error>(())
    });

    // Wait for tasks (ignore exact errors for now)
    let _ = tokio::join!(reader, ev_forward, writer);
    core_tx.send(CoreMsg::UnregisterPeer { peer }).await.ok();
    Ok(())
}
