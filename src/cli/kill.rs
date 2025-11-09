use rust_args_parser as ap;

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
            for _ in 2000..3000 {
                let win_id = alloc.allocate(WindowId::new);
                println!("window id: {}", win_id);
            }
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
