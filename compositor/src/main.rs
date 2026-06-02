mod comp;
mod drm_be;
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

    log::info!("zeroday-comp v4.3.0 starting");
    log::info!("  Screen #1 (LCD): {} @ {}fps — controls, GUI launcher", args.resolution, args.fps);
    log::info!("  Screen #2 (HDMI): 1920x1080 @ {}fps — content display (hotplug auto-detect)", args.hdmi_fps);
    log::info!("  client: {}", args.client);

    panic_handler::install();

    let _hotplug = hdmi::hdmi_hotplug_thread();
    let hdmi_on = hdmi::is_hdmi_connected();
    log::info!("HDMI status: {}", if hdmi_on { "CONNECTED — Screen #2 active" } else { "disconnected — LCD-only mode" });

    let app_data = comp::AppData {
        client_cmd: args.client,
        client_args: args.client_args,
        no_cursor: args.no_cursor,
        target_fps: args.fps,
        drm_path: args.drm_device,
        hdmi_fps: args.hdmi_fps,
        hdmi_auto: args.hdmi_auto,
        input_handler: input::InputHandler::new(),
    };

    if let Err(e) = comp::run(app_data) {
        log::error!("zeroday-comp error: {:?}", e);
        log::warn!("Falling back — cage or Xorg+i3 will take over via systemd OnFailure=");
    }
}