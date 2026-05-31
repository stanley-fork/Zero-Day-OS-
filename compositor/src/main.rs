mod hdmi;
mod input;
mod panic_handler;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    name = "zeroday-comp",
    version,
    about = "Wayland compositor for ZERO-DAY OS on M5Stack Cardputer Zero"
)]
struct Args {
    #[arg(long, default_value = "cyber_launcher")]
    client: String,
    #[arg(long)]
    client_args: Option<String>,
    #[arg(long, default_value = "320x170")]
    resolution: String,
    #[arg(long, default_value = "30")]
    fps: u32,
    #[arg(long, default_value = "/dev/dri/card0")]
    drm_device: String,
    #[arg(long, default_value = "true")]
    no_cursor: bool,
    #[arg(long)]
    verbose: bool,
    #[arg(long, default_value = "30")]
    hdmi_fps: u32,
    #[arg(long, default_value = "true")]
    hdmi_auto: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }

    log::info!("zeroday-comp starting (development build)");
    log::info!("  LCD: {} @ {}fps", args.resolution, args.fps);
    log::info!("  HDMI: 1920x1080 @ {}fps (hotplug auto-detect)", args.hdmi_fps);
    log::info!("  client: {}", args.client);

    panic_handler::install();

    let _hotplug = hdmi::hdmi_hotplug_thread();
    let hdmi_on = hdmi::is_hdmi_connected();
    log::info!("HDMI status: {}", if hdmi_on { "CONNECTED" } else { "disconnected" });

    log::warn!("zeroday-comp is a work-in-progress — falling back to cage for now");
    log::info!("Starting client directly with Wayland environment variables...");

    let cmd = &args.client;
    let socket_name = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());

    let args_vec: Vec<&str> = args.client_args
        .as_deref()
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    let hdmi_env = if hdmi_on { "1" } else { "0" };

    log::info!("Executing: {} {:?}", cmd, args_vec);

    let mut child = std::process::Command::new(cmd)
        .args(&args_vec)
        .env("WAYLAND_DISPLAY", &socket_name)
        .env("SDL_VIDEODRIVER", "wayland")
        .env("SDL_RENDER_DRIVER", "opengles2")
        .env("PYGAME_HIDE_SUPPORT_PROMPT", "1")
        .env("ZERODAY_HDMI", hdmi_env)
        .env("ZERODAY_LCD_WIDTH", "320")
        .env("ZERODAY_LCD_HEIGHT", "170")
        .env("ZERODAY_HDMI_WIDTH", "1920")
        .env("ZERODAY_HDMI_HEIGHT", "1080")
        .env("ZERODAY_HDMI_FPS", args.hdmi_fps.to_string())
        .spawn()
        .expect("failed to start client");

    let status = child.wait().expect("failed to wait for client");
    log::info!("Client exited with status: {}", status);
}