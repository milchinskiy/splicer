use rust_args_parser as ap;

#[derive(Default)]
pub struct ServerContext {
    foreground: bool,
}

pub fn command<'a>() -> ap::CmdSpec<'a, super::Context> {
    ap::CmdSpec::new(
        Some("server"),
        Some(|_, _ctx: &mut super::Context| {
            unimplemented!();
        }),
    )
    .desc("Start splicer server")
    .opts([ap::OptSpec::new("foreground", |_, ctx: &mut super::Context| {
        ctx.server.foreground = true;
        Ok(())
    })
    .short('f')
    .flag()
    .help("Run in foreground")])
}
