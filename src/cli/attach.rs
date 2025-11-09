use rust_args_parser as ap;

#[derive(Default)]
pub struct AttachContext {
    window: Option<String>,
    pane: Option<String>,
}

pub fn command<'a>() -> ap::CmdSpec<'a, super::Context> {
    ap::CmdSpec::new(Some("attach"), None)
        .desc("Attach to a session")
        .opts([
            ap::OptSpec::new("window", |s, ctx: &mut super::Context| {
                ctx.attach.window = s.map(|s| s.to_string());
                Ok(())
            })
            .short('w')
            .required()
            .metavar("WINDOW")
            .help("Window name or ID"),
            ap::OptSpec::new("pane", |s, ctx: &mut super::Context| {
                ctx.attach.pane = s.map(|s| s.to_string());
                Ok(())
            })
            .short('p')
            .required()
            .metavar("PANE")
            .help("Pane name or ID"),
        ])
        .pos([ap::PosSpec::new("SESSION").one().desc("Session name or ID")])
}
