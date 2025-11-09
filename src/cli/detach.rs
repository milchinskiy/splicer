use rust_args_parser as ap;

#[derive(Default)]
pub struct DetachContext {
    session: Option<String>,
    window: Option<String>,
    pane: Option<String>,
    peer: Option<String>,
    all_peers: bool,
}

pub fn command<'a>() -> ap::CmdSpec<'a, super::Context> {
    ap::CmdSpec::new(
        Some("detach"),
        Some(|args, ctx: &mut super::Context| {
            ctx.detach.session = args.first().map(|s| s.to_string());
            Ok(())
        }),
    )
    .desc("Detach from session")
    .opts([
        ap::OptSpec::new("window", |s, ctx: &mut super::Context| {
            ctx.detach.window = s.map(|s| s.to_string());
            Ok(())
        })
        .short('w')
        .at_most_one(1)
        .required()
        .metavar("WINDOW")
        .help("Detach self from window"),
        ap::OptSpec::new("pane", |s, ctx: &mut super::Context| {
            ctx.detach.pane = s.map(|s| s.to_string());
            Ok(())
        })
        .short('p')
        .at_most_one(1)
        .required()
        .metavar("PANE")
        .help("Detach self from pane"),
        ap::OptSpec::new("peer", |s, ctx: &mut super::Context| {
            ctx.detach.peer = s.map(|s| s.to_string());
            Ok(())
        })
        .short('P')
        .at_most_one(2)
        .required()
        .metavar("PEER")
        .help("Detach peer from target"),
        ap::OptSpec::new("all-peers", |_, ctx: &mut super::Context| {
            ctx.detach.all_peers = true;
            Ok(())
        })
        .short('a')
        .at_most_one(2)
        .flag()
        .help("Detach all peers from target"),
    ])
    .pos([ap::PosSpec::new("SESSION").range(0, 1).desc("Session name or ID")])
}
