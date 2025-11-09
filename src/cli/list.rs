use rust_args_parser as ap;

#[derive(Default)]
pub struct ListContext {
    sessions: bool,
    windows: bool,
    panes: bool,
    peers: bool,
    session: Option<String>,
    window: Option<String>,
}

pub fn command<'a>() -> ap::CmdSpec<'a, super::Context> {
    ap::CmdSpec::new(Some("ls"), Some(|_, _ctx: &mut super::Context| unimplemented!()))
        .aliases(["ls"])
        .desc("List sessions/windows/panes/peers")
        .opts([
            ap::OptSpec::new("sessions", |_, ctx: &mut super::Context| {
                ctx.list.sessions = true;
                Ok(())
            })
            .help("List sessions")
            .flag(),
            ap::OptSpec::new("windows", |_, ctx: &mut super::Context| {
                ctx.list.windows = true;
                Ok(())
            })
            .help("List windows")
            .flag(),
            ap::OptSpec::new("panes", |_, ctx: &mut super::Context| {
                ctx.list.panes = true;
                Ok(())
            })
            .help("List panes")
            .flag(),
            ap::OptSpec::new("peers", |_, ctx: &mut super::Context| {
                ctx.list.peers = true;
                Ok(())
            })
            .help("List peers")
            .flag(),
            ap::OptSpec::new("all", |_, ctx: &mut super::Context| {
                ctx.list.sessions = true;
                ctx.list.windows = true;
                ctx.list.panes = true;
                ctx.list.peers = true;
                Ok(())
            })
            .help("List all")
            .flag(),
            ap::OptSpec::new("session", |s, ctx: &mut super::Context| {
                ctx.list.session = Some(s.unwrap_or("").to_string());
                Ok(())
            })
            .short('s')
            .help("Of session")
            .required(),
            ap::OptSpec::new("window", |s, ctx: &mut super::Context| {
                ctx.list.window = Some(s.unwrap_or("").to_string());
                Ok(())
            })
            .short('w')
            .help("Of window")
            .flag(),
        ])
}
