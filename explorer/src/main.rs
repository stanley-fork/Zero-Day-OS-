use clap::Parser;

mod app;
mod ui;
mod fsops;
mod hexview;
mod search;
mod bookmarks;
mod input;

#[derive(Parser, Debug)]
#[command(name = "zeroday-fm", about = "File explorer for ZERO-DAY OS (320x170, 46-key, no mouse)", version)]
struct Args {
    #[arg(default_value = ".")]
    path: String,
    #[arg(short, long, help = "Show hidden files")]
    hidden: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();
    let start_path = std::path::PathBuf::from(&args.path);
    let start_dir = if start_path.is_dir() {
        start_path
    } else {
        std::env::current_dir()?
    };
    let mut app = app::App::new(start_dir, args.hidden)?;
    ui::run(&mut app)
}