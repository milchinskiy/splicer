use rust_args_parser as ap;

mod attach;
mod detach;
mod kill;
mod list;
mod new;
mod server;

#[derive(Default)]
pub struct Context {
    json: bool,
    quiet: bool,
    socket: Option<std::path::PathBuf>,
    server: server::ServerContext,
    new: new::NewContext,
    attach: attach::AttachContext,
    detach: detach::DetachContext,
    kill: kill::KillContext,
    list: list::ListContext,
}

/// CLI entry point
/// # Errors [`Error`]
pub fn run() -> splicer::Result {
    let env = ap::Env::new("splicer").version(env!("CARGO_PKG_VERSION")).auto_help(true).auto_color();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let argv: Vec<&str> = argv.iter().map(std::string::String::as_str).collect();
    let mut ctx = Context::default();
    let socket_path = default_socket_path().to_string_lossy().to_string();

    let root = ap::CmdSpec::new(None, None)
        .desc("Splicer terminal multiplexer")
        .subs([
            new::command(),
            attach::command(),
            detach::command(),
            list::command(),
            server::command(),
            kill::command(),
        ])
        .opts([
            ap::OptSpec::new("json", |_, ctx: &mut Context| {
                ctx.json = true;
                Ok(())
            })
            .short('j')
            .flag()
            .help("JSON output"),
            ap::OptSpec::new("quiet", |_, ctx: &mut Context| {
                ctx.quiet = true;
                Ok(())
            })
            .short('q')
            .flag()
            .help("Quiet mode"),
            ap::OptSpec::new("socket", |s, ctx: &mut Context| {
                ctx.socket = s.map(std::path::PathBuf::from);
                Ok(())
            })
            .help("Socket path")
            .default(&socket_path)
            .required()
            .metavar("PATH")
            .env("SPLICER_SOCKET"),
        ]);

    match ap::dispatch(&env, &root, &argv, &mut ctx) {
        Ok(()) => Ok(()),
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn default_socket_path() -> std::path::PathBuf {
    let path = std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

    path.join("splicer.sock")
}
