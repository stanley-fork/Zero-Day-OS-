mod pty;
mod term;
mod render;
mod fn_keys;
mod status_bar;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    name = "zeroday-term",
    version,
    about = "Terminal emulator for ZERO-DAY OS (320x170, 46-key)"
)]
struct Args {
    #[arg(short, long, default_value = "/bin/bash")]
    shell: String,
    #[arg(short, long)]
    command: Option<String>,
    #[arg(long, default_value = "8")]
    font_size: u16,
    #[arg(long, default_value = "320")]
    width: u32,
    #[arg(long, default_value = "170")]
    height: u32,
    #[arg(long)]
    no_status: bool,
    #[arg(long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }

    log::info!("zeroday-term starting ({}x{}, font {})", args.width, args.height, args.font_size);

    let mut terminal = term::Terminal::new(&args);
    if let Err(e) = terminal.run() {
        log::error!("Terminal error: {}", e);
        std::process::exit(1);
    }
}