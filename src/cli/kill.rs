use rust_args_parser as ap;
use splicer::server::pane::TermSize;
use splicer::server::state::ServerState;

#[derive(Default)]
pub struct KillContext {
    force: bool,
    session: Option<String>,
    window: Option<String>,
    pane: Option<String>,
}

pub fn command<'a>() -> ap::CmdSpec<'a, super::Context> {
    ap::CmdSpec::new(
        Some("kill"),
        Some(|_, _ctx: &mut super::Context| {
            use splicer::server::window::WindowId;
            let mut alloc = splicer::server::IdAllocator::default();
            for _ in 2000..2005 {
                let win_id = alloc.allocate(WindowId::new);
                println!("window id: {}", win_id);
                let win = splicer::server::window::Window::new(win_id, "test");
                println!("window: {}", win);
            }

            let mut st = ServerState::new();
            let sid = st.new_session("work");
            let wid = st.new_window(sid, "main")?;
            let pid = st.new_pane(sid, wid, "shell", TermSize::new(120, 34))?;
            println!("state: {}; sid={}; wid={}; pid={}", st, sid, wid, pid);

            Ok(())
        }),
    )
    .desc("Destroy a session/window/pane")
    .opts([ap::OptSpec::new("force", |_, ctx: &mut super::Context| {
        ctx.kill.force = true;
        Ok(())
    })
    .short('f')
    .flag()
    .help("Force kill")])
    .subs([
        ap::CmdSpec::new(
            Some("session"),
            Some(|args, ctx: &mut super::Context| {
                ctx.kill.session = args.first().map(|s| s.to_string());
                Ok(())
            }),
        )
        .pos([ap::PosSpec::new("SESSION").one().desc("Session name or ID")]),
        ap::CmdSpec::new(
            Some("window"),
            Some(|args, ctx: &mut super::Context| {
                ctx.kill.window = args.first().map(|s| s.to_string());
                Ok(())
            }),
        )
        .pos([ap::PosSpec::new("WINDOW").one().desc("Window name or ID")]),
        ap::CmdSpec::new(
            Some("pane"),
            Some(|args, ctx: &mut super::Context| {
                ctx.kill.pane = args.first().map(|s| s.to_string());
                Ok(())
            }),
        )
        .pos([ap::PosSpec::new("PANE").one().desc("Pane name or ID")]),
    ])
}
