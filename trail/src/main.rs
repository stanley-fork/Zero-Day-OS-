mod breadcrumb;
mod config;
mod guide;
mod scanner;
mod store;

use clap::Parser;
use config::Config;

#[derive(Parser, Debug)]
#[clap(
    name = "zeroday-trail",
    version,
    about = "Breadcrumb navigation daemon for ZERO-DAY OS — WiFi fingerprint waypointing and exit guidance"
)]
struct Args {
    #[arg(short, long, default_value = "wlan0")]
    iface: Option<String>,

    #[arg(short, long)]
    interval: Option<u64>,

    #[arg(short, long)]
    threshold: Option<u32>,

    #[arg(long)]
    no_overwatch: bool,

    #[arg(short, long)]
    quiet: bool,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    Start,
    Mark { tag: String },
    Exit,
    Pause,
    Resume,
    Stats,
    Dump,
    Clear,
    Status,
}

enum Mode {
    Drop,
    Exit,
    Paused,
}

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(if args.quiet { "warn" } else { "info" })).init();

    if let Some(cmd) = &args.command {
        run_command(cmd, &args);
        return;
    }

    run_daemon(&args);
}

fn run_command(cmd: &Command, args: &Args) {
    let cfg = Config::from_args(args.iface.clone(), args.interval, args.threshold, args.no_overwatch, args.quiet);
    let store = store::BreadcrumbStore::new();

    match cmd {
        Command::Start => {
            send_signal("start");
        }
        Command::Mark { tag } => {
            if let Ok(pid) = read_pid() {
                let marker_path = std::path::Path::new("/tmp/trail-mark");
                std::fs::write(marker_path, tag).ok();
                println!("Tagged waypoint: {}", tag);
            } else {
                eprintln!("Trail daemon not running — start it first with: trail-ctl start");
                std::process::exit(1);
            }
        }
        Command::Exit => {
            if let Ok(pid) = read_pid() {
                let mode_path = std::path::Path::new("/tmp/trail-mode");
                std::fs::write(mode_path, "exit").ok();
                println!("Exit guidance activated — follow the breadcrumbs out");
            } else {
                eprintln!("Trail daemon not running — start it first: trail start");
                std::process::exit(1);
            }
        }
        Command::Pause => {
            let mode_path = std::path::Path::new("/tmp/trail-mode");
            std::fs::write(mode_path, "pause").ok();
            println!("Trail paused — breadcrumbs stopped");
        }
        Command::Resume => {
            let mode_path = std::path::Path::new("/tmp/trail-mode");
            std::fs::write(mode_path, "drop").ok();
            println!("Trail resumed — dropping breadcrumbs");
        }
        Command::Stats => {
            match store.load_all() {
                Ok(bcs) => {
                    let count = bcs.len();
                    let tagged = bcs.iter().filter(|b| b.tag.is_some()).count();
                    let first = bcs.first().map(|b| b.timestamp).unwrap_or(0);
                    let last = bcs.last().map(|b| b.timestamp).unwrap_or(0);
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    let duration = now.saturating_sub(first);
                    let hrs = duration / 3600;
                    let mins = (duration % 3600) / 60;
                    println!("Trail stats:");
                    println!("  Breadcrumbs:  {}", count);
                    println!("  Tagged:        {}", tagged);
                    println!("  Duration:      {}h{}m", hrs, mins);
                    println!("  First drop:    {}s ago", now.saturating_sub(first));
                    println!("  Last drop:     {}s ago", now.saturating_sub(last));
                    if let Some(bc) = bcs.last() {
                        println!("  Last AP count: {}", bc.fingerprints.len());
                    }
                }
                Err(e) => eprintln!("Error reading breadcrumbs: {}", e),
            }
        }
        Command::Dump => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            let path = format!("/opt/cardputer/trail/exports/trail-{}.gpx", now);
            if let Err(e) = std::fs::create_dir_all("/opt/cardputer/trail/exports") {
                eprintln!("Cannot create export dir: {}", e);
                std::process::exit(1);
            }
            match store.export_gpx(&path) {
                Ok(()) => println!("Exported trail to {}", path),
                Err(e) => eprintln!("Export failed: {}", e),
            }
        }
        Command::Clear => {
            match store.clear_today() {
                Ok(()) => println!("Today's breadcrumbs cleared"),
                Err(e) => eprintln!("Clear failed: {}", e),
            }
        }
        Command::Status => {
            let running = read_pid().is_ok();
            let mode = std::fs::read_to_string("/tmp/trail-mode").unwrap_or_else(|_| "unknown".into());
            println!("Trail status: {}", if running { "running" } else { "stopped" });
            println!("Mode: {}", mode.trim());
            if running {
                if let Ok(bcs) = store.load_today() {
                    println!("Today's breadcrumbs: {}", bcs.len());
                }
            }
        }
    }
}

fn run_daemon(args: &Args) {
    let cfg = Config::from_args(args.iface.clone(), args.interval, args.threshold, args.no_overwatch, args.quiet);
    let quiet = cfg.quiet;

    if !quiet { log::info!("zeroday-trail starting on {}", cfg.wifi_iface); }

    let scanner = scanner::WifiScanner::new(&cfg.wifi_iface);
    let store = store::BreadcrumbStore::with_dir(&cfg.data_dir);
    let mut journal = breadcrumb::BreadcrumbJournal::with_capacity(cfg.max_breadcrumbs);
    let mut overwatch = guide::OverwatchDetector::new();
    let guide = guide::GuideEngine::new(cfg.match_threshold);

    if let Err(e) = store.ensure_dir() {
        log::error!("Cannot create data directory: {}", e);
        std::process::exit(1);
    }

    if let Ok(bcs) = store.load_all() {
        for bc in &bcs {
            journal.drop(bc.clone());
        }
        if !quiet { log::info!("Loaded {} existing breadcrumbs", bcs.len()); }
        overwatch.learn_baseline(&bcs);
    }

    if let Err(e) = config::write_default_config() {
        log::warn!("Cannot write default config: {}", e);
    }

    write_pid();
    write_mode(Mode::Drop);

    let mut mode = Mode::Drop;
    let mut threat_level = breadcrumb::ThreatLevel::None;
    let tick = std::time::Duration::from_secs(cfg.scan_interval_secs);

    if !quiet { log::info!("Trail daemon running (interval={}s, threshold={}%)", cfg.scan_interval_secs, cfg.match_threshold); }

    loop {
        mode = read_mode();

        match mode {
            Mode::Paused => {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            Mode::Drop | Mode::Exit => {}
        }

        if let Ok(current) = scanner.current_fingerprint() {
            match mode {
                Mode::Drop => {
                    if !quiet { log::info!("Drop: {} APs seen", current.fingerprints.len()); }
                    if let Err(e) = store.append(&current) {
                        log::warn!("Store error: {}", e);
                    }
                    journal.drop(current.clone());

                    if cfg.overwatch_enabled {
                        if cfg.evil_twin_detect {
                            if let Some(level) = overwatch.detect_evil_twin(&current) {
                                threat_level = level;
                                if !quiet { log::warn!("Overwatch: evil twin detected ({})", level); }
                            }
                        }
                        if cfg.new_ap_watch {
                            let new_level = overwatch.detect_new_aps(&current);
                        if new_level > threat_level {
                                threat_level = new_level;
                            }
                        }

                        let ssid = scanner.connected_ssid();
                        let bssid = scanner.connected_bssid();
                        overwatch.set_connected(ssid, bssid);
                    }

                    write_status(&journal, &threat_level);
                }
                Mode::Exit => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

                    if let Ok(current) = scanner.current_fingerprint() {
                        let trail = journal.trail();
                        let guidance = guide.navigate(&current, trail, now);

                        if !quiet {
                            let line = guide::GuideEngine::format_guidance(&guidance, 40);
                            log::info!("EXIT: {}", line);
                        }

                        write_exit_guidance(&guidance);
                    }
                }
                Mode::Paused => unreachable!(),
            }
        }

        if let Ok(tag) = std::fs::read_to_string("/tmp/trail-mark") {
            let _ = std::fs::remove_file("/tmp/trail-mark");
            if let Some(last) = journal.last() {
                let mut bc = last.clone();
                bc.tag = Some(tag.trim().to_string());
                let _ = store.append(&bc);
            }
        }

        let decay_before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
            .saturating_sub(cfg.decay_hours * 3600);
        journal.prune_older_than(decay_before);

        std::thread::sleep(tick);
    }
}

fn write_pid() {
    let pid = std::process::id();
    let _ = std::fs::write("/tmp/trail.pid", pid.to_string());
}

fn read_pid() -> Result<u32, String> {
    let content = std::fs::read_to_string("/tmp/trail.pid")
        .map_err(|e| format!("Trail daemon not running: {}", e))?;
    let pid: u32 = content.trim().parse()
        .map_err(|e| format!("Invalid PID: {}", e))?;
    let exists = std::path::Path::new(&format!("/proc/{}", pid)).exists();
    if exists { Ok(pid) } else { Err("Trail daemon not running".into()) }
}

fn write_mode(mode: Mode) {
    let s = match mode {
        Mode::Drop => "drop",
        Mode::Exit => "exit",
        Mode::Paused => "pause",
    };
    let _ = std::fs::write("/tmp/trail-mode", s);
}

fn read_mode() -> Mode {
    let content = std::fs::read_to_string("/tmp/trail-mode").unwrap_or_else(|_| "drop".into());
    match content.trim() {
        "exit" => Mode::Exit,
        "pause" => Mode::Paused,
        _ => Mode::Drop,
    }
}

fn write_status(journal: &breadcrumb::BreadcrumbJournal, threat: &breadcrumb::ThreatLevel) {
    let count = journal.len();
    let line = format!("[TRAIL ● {}crumbs] [{}]", count, threat);
    let _ = std::fs::write("/tmp/trail-status", line);
}

fn write_exit_guidance(guidance: &breadcrumb::ExitGuidance) {
    let tag = guidance.waypoint_tag.as_deref().unwrap_or("waypoint");
    let line = match guidance.direction {
        breadcrumb::ExitDirection::Here => format!("[TRAIL ✓ EXIT {}%]", guidance.match_pct),
        breadcrumb::ExitDirection::Lost => format!("[TRAIL !! LOST {}%]", guidance.match_pct),
        _ => format!("[TRAIL {} {}% → {}]", guidance.direction, guidance.match_pct, tag),
    };
    let _ = std::fs::write("/tmp/trail-status", line);
}

fn send_signal(action: &str) {
    let _ = std::fs::write("/tmp/trail-mode", action);
    println!("Sent '{}' signal to trail daemon", action);
}