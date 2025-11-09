use rust_args_parser as ap;

#[derive(Default)]
pub struct NewContext {
    session: Option<String>,
    cwd: Option<std::path::PathBuf>,
    title: Option<String>,
}

pub fn command<'a>() -> ap::CmdSpec<'a, super::Context> {
    ap::CmdSpec::new(Some("new"), Some(|_, _ctx: &mut super::Context| unimplemented!()))
        .desc("Create a new session")
        .opts([
            ap::OptSpec::new("session", |s, ctx: &mut super::Context| {
                ctx.new.session = s.map(|s| s.to_string());
                Ok(())
            })
            .short('s')
            .help("Session name")
            .required()
            .metavar("NAME"),
            ap::OptSpec::new("cwd", |s, ctx: &mut super::Context| {
                ctx.new.cwd = s.map(std::path::PathBuf::from);
                Ok(())
            })
            .short('c')
            .help("Working directory")
            .required()
            .metavar("PATH"),
            ap::OptSpec::new("title", |s, ctx: &mut super::Context| {
                ctx.new.title = s.map(|s| s.to_string());
                Ok(())
            })
            .short('t')
            .help("Window title")
            .metavar("TITLE")
            .required(),
        ])
}
