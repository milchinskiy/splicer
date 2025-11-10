use super::*;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::thread;
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch};

const READ_CHUNK: usize = 16 * 1024; // 16 KiB

pub(crate) struct Inner {
    master: Mutex<Box<dyn MasterPty + Send>>,               // writer+resize target
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>, // shared child handle
    in_tx: mpsc::Sender<Vec<u8>>,                           // buffered stdin pipeline
    out_taps: Arc<Mutex<Vec<mpsc::Sender<ByteChunk>>>>,     // fan‑out taps (Arc for sharing)
    exit_tx: watch::Sender<Option<ExitStatus>>,             // exit watcher
}

pub(super) fn spawn(program: Program, cfg: PtyConfig) -> Result<PtyHandle> {
    // `PtySystem` trait must be in scope for `.openpty(...)`
    let pty_system = NativePtySystem::default();
    let pair: PtyPair = pty_system
        .openpty(PtySize { rows: cfg.rows, cols: cfg.cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|_| PtyError::OpenFailed)?;

    // Prepare command
    let mut cmd = match program {
        Program::Shell => {
            let shell = std::env::var_os("SHELL").unwrap_or_else(|| "/bin/sh".into());
            CommandBuilder::new(shell)
        }
        Program::Argv { argv } => {
            let mut it = argv.into_iter();
            let Some(prog) = it.next() else {
                return Err(PtyError::InvalidArgs("empty argv"));
            };
            let mut b = CommandBuilder::new(prog);
            for a in it {
                b.arg(a);
            }
            b
        }
    };

    cmd.env("TERM", cfg.term.as_deref().unwrap_or("xterm-256color"));
    if let Some(ref cwd) = cfg.cwd {
        cmd.cwd(cwd);
    }
    for (k, v) in &cfg.env {
        cmd.env(k, v);
    }

    // Spawn the child attached to the slave
    let child = pair.slave.spawn_command(cmd).map_err(|_| PtyError::SpawnFailed)?;

    // Master PTY handles
    let master: Box<dyn MasterPty + Send> = pair.master; // keep for resize
    let mut reader = master
        .try_clone_reader()
        .map_err(|e| PtyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?; // portable-pty uses anyhow::Error

    // Writer: wrap in Arc<Mutex<...>> so we can offload each write into spawn_blocking
    let writer = Arc::new(Mutex::new(
        master
            .take_writer()
            .map_err(|e| PtyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?,
    ));

    // Writer pipeline (stdin): async recv → blocking write in blocking pool
    let (in_tx, mut in_rx) = mpsc::channel::<Vec<u8>>(256);
    {
        let writer = writer.clone();
        tokio::spawn(async move {
            while let Some(buf) = in_rx.recv().await {
                let writer = writer.clone();
                let res = tokio::task::spawn_blocking(move || {
                    let mut w = writer.lock().expect("writer mutex poisoned");
                    w.write_all(&buf)
                })
                .await;
                match res {
                    Ok(Ok(())) => {}
                    _ => break,
                }
            }
        });
    }

    // Reader fan‑out: dedicated OS thread (blocking `read`) → MPSC taps
    let (exit_tx, _exit_rx) = watch::channel::<Option<ExitStatus>>(None);
    let out_taps = Arc::new(Mutex::new(Vec::<mpsc::Sender<ByteChunk>>::new()));

    // Shared child for wait(), cloned into the reader thread and stored in Inner
    let child_shared: Arc<Mutex<Box<dyn portable_pty::Child + Send>>> = Arc::new(Mutex::new(child));
    let child_for_wait = child_shared.clone();
    let exit_tx_thr = exit_tx.clone();
    let out_taps_thr = out_taps.clone();

    thread::spawn(move || {
        let mut buf = vec![0u8; READ_CHUNK];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk: ByteChunk = Arc::from(&buf[..n]);
                    let mut dead = Vec::new();
                    if let Ok(mut taps) = out_taps_thr.lock() {
                        for (i, tx) in taps.iter_mut().enumerate() {
                            if tx.try_send(chunk.clone()).is_err() {
                                dead.push(i);
                            }
                        }
                        for i in dead.into_iter().rev() {
                            taps.remove(i);
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        // Wait for child and announce exit
        if let Ok(mut c) = child_for_wait.lock() {
            let status = c.wait().ok().map(|s| {
                let code = s.exit_code();
                let sig: Option<String> = s.signal().map(|name| name.to_string());
                ExitStatus { code, signal: sig }
            });
            let _ = exit_tx_thr.send(status);
        } else {
            let _ = exit_tx_thr.send(Some(ExitStatus { code: 0, signal: None }));
        }
    });

    // Build handle (keep master for future resizes)
    let inner = Arc::new(Inner { master: Mutex::new(master), child: child_shared, in_tx, out_taps, exit_tx });

    Ok(PtyHandle { inner })
}

pub(super) async fn write(h: &PtyHandle, bytes: &[u8]) -> Result<usize> {
    h.inner
        .in_tx
        .send(bytes.to_vec())
        .await
        .map_err(|_| PtyError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pty stdin closed")))?;
    Ok(bytes.len())
}

pub(super) fn resize(h: &PtyHandle, cols: u16, rows: u16) -> Result<()> {
    let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
    let master = h.inner.master.lock().map_err(|_| PtyError::ResizeFailed)?;
    master.resize(size).map_err(|_| PtyError::ResizeFailed)
}

pub(super) fn subscribe(h: &PtyHandle) -> OutputRx {
    let (tx, rx) = mpsc::channel::<ByteChunk>(512);
    if let Ok(mut taps) = h.inner.out_taps.lock() {
        taps.push(tx);
    }
    rx
}

pub(super) fn signal(h: &PtyHandle, sig: Sig) -> Result<()> {
    match sig {
        Sig::Int => {
            h.inner
                .in_tx
                .try_send(vec![0x03])
                .map_err(|_| PtyError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pty stdin closed")))?;
            Ok(())
        }
        Sig::Hup => Ok(()),
        Sig::Term | Sig::Kill => {
            let mut child = h.inner.child.lock().map_err(|_| PtyError::WaitFailed)?;
            child.kill().map_err(|_| PtyError::Io(std::io::Error::from(std::io::ErrorKind::Other)))
        }
    }
}

pub(super) fn exit_watch(h: &PtyHandle) -> watch::Receiver<Option<ExitStatus>> {
    h.inner.exit_tx.subscribe()
}
