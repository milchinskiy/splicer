use super::proto::*;
use super::wire::*;
use crate::{Error, Result};
use rmp_serde::{from_slice, to_vec};
use std::sync::Arc;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot},
};

pub struct IpcClient {
    w: OwnedWriteHalf,
    ev_rx: mpsc::Receiver<Event>,
    // Single-flight waiter: next Response will be delivered here by the reader task
    resp_waiter: Arc<tokio::sync::Mutex<Option<oneshot::Sender<Response>>>>,
}

impl IpcClient {
    pub async fn connect(path: &str) -> Result<Self> {
        let sock = UnixStream::connect(path).await?;
        let (mut r, mut w): (OwnedReadHalf, OwnedWriteHalf) = sock.into_split();

        // Hello
        let hello = Hello { client_api_major: 1, features: 0 };
        let bytes = to_vec(&hello).unwrap();
        let hdr = FrameHeader { api_major: 1, kind: Kind::Request, schema_id: 0, len: bytes.len() as u32 };
        write_payload(&mut w, hdr, &bytes).await?;
        let (_hdr, _ack) = read_payload(&mut r).await?; // could validate majors/features

        // Event & response routing
        let (ev_tx, ev_rx) = mpsc::channel(256);
        let resp_waiter: Arc<tokio::sync::Mutex<Option<oneshot::Sender<Response>>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let resp_waiter_reader = resp_waiter.clone();

        // Single reader/demux task owns the read half
        tokio::spawn(async move {
            while let Ok((hdr, bytes)) = read_payload(&mut r).await {
                match hdr.kind {
                    Kind::Event => {
                        if let Ok(ev) = from_slice::<Event>(&bytes) {
                            let _ = ev_tx.send(ev).await;
                        }
                    }
                    Kind::Response => {
                        if let Ok(resp) = from_slice::<Response>(&bytes) {
                            if let Some(tx) = resp_waiter_reader.lock().await.take() {
                                let _ = tx.send(resp);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self { w, ev_rx, resp_waiter })
    }

    pub async fn request(&mut self, req: Request) -> Result<Response> {
        // Install single-flight waiter
        let (tx, rx) = oneshot::channel();
        *self.resp_waiter.lock().await = Some(tx);

        let bytes = to_vec(&req).unwrap();
        let hdr = FrameHeader { api_major: 1, kind: Kind::Request, schema_id: 1, len: bytes.len() as u32 };
        write_payload(&mut self.w, hdr, &bytes).await?;

        match rx.await {
            Ok(resp) => Ok(resp),
            Err(_) => Err(Error::Ipc("connection closed".into())),
        }
    }

    /// Consume and return the event stream receiver.
    pub fn take_events(&mut self) -> mpsc::Receiver<Event> {
        std::mem::replace(&mut self.ev_rx, mpsc::channel(1).1)
    }
}
