mod cli;

fn main() -> splicer::Result {
    init_log();
    cli::run()
}

fn init_log() {
    #[cfg(debug_assertions)]
    {
        rustlog::set_level(rustlog::Level::Trace);
        rustlog::set_show_file_line(true);
        rustlog::set_show_thread_id(true);
        rustlog::banner!();
    }
    #[cfg(not(debug_assertions))]
    {
        rustlog::set_level(rustlog::Level::Info);
        rustlog::set_show_file_line(false);
        rustlog::set_show_thread_id(false);
    }
}

