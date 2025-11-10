use splicer::pty::{self, Program, PtyConfig};
use tokio::time::{Duration, timeout};

#[tokio::test]
async fn spawn_shell_and_echo() {
    let handle = pty::spawn(Program::Shell, PtyConfig { cols: 80, rows: 24, cwd: None, env: vec![], term: None })
        .expect("spawn shell");

    let mut rx = handle.subscribe();
    handle.write(b"echo hi\n").await.unwrap();

    // Read until we see "hi" or time out
    let mut buf = Vec::new();
    loop {
        // don't block forever; bail if no chunk in 2s
        let ch = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timed out waiting for shell output")
            .expect("pty output channel closed");
        buf.extend_from_slice(&ch);
        if String::from_utf8_lossy(&buf).contains("hi") {
            break;
        }
    }

    // Terminate the shell so the reader thread exits cleanly
    handle.write(b"exit\n").await.unwrap();
    let mut w = handle.exit_watch();
    timeout(Duration::from_secs(2), async {
        while w.borrow().is_none() {
            w.changed().await.unwrap();
        }
    })
    .await
    .expect("shell did not exit");
}

#[tokio::test]
async fn spawn_argv_and_exit_status() {
    let handle = pty::spawn(
        Program::Argv { argv: vec!["/usr/bin/env".into(), "printf".into(), "%s".into(), "OK".into()] },
        PtyConfig { cols: 80, rows: 24, cwd: None, env: vec![], term: None },
    )
    .expect("spawn argv");

    let mut rx = handle.subscribe();
    let mut out = Vec::new();
    while let Some(ch) = rx.recv().await {
        out.extend_from_slice(&ch);
        if out.len() >= 2 {
            break;
        }
    }
    assert_eq!(String::from_utf8_lossy(&out), "OK");

    // Wait for child exit via watch::Receiver
    let mut w = handle.exit_watch();
    timeout(Duration::from_secs(2), async {
        while w.borrow().is_none() {
            w.changed().await.unwrap();
        }
    })
    .await
    .expect("argv child did not exit");
}

#[tokio::test]
async fn run_noninteractive() {
    let h = pty::spawn(
        Program::Argv { argv: vec!["/bin/sh".into(), "-lc".into(), "printf hi".into()] },
        PtyConfig { cols: 80, rows: 24, cwd: None, env: vec![], term: None },
    )
    .unwrap();

    let mut rx = h.subscribe();
    let mut out = Vec::new();
    // Read until we see the expected bytes; the child exits immediately after
    while let Some(ch) = rx.recv().await {
        out.extend_from_slice(&ch);
        if out.ends_with(b"hi") {
            break;
        }
    }

    // Wait for exit without a timeout (child is guaranteed to terminate)
    let mut w = h.exit_watch();
    while w.borrow().is_none() {
        w.changed().await.unwrap();
    }
}
