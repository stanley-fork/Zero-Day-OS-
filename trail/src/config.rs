use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = "/etc/zeroday/trail";
const DATA_DIR: &str = "/opt/cardputer/trail/breadcrumbs";

#[derive(Debug, Clone)]
pub struct Config {
    pub wifi_iface: String,
    pub scan_interval_secs: u64,
    pub match_threshold: u32,
    pub max_breadcrumbs: usize,
    pub decay_hours: u64,
    pub data_dir: String,
    pub overwatch_enabled: bool,
    pub evil_twin_detect: bool,
    pub new_ap_watch: bool,
    pub quiet: bool,
}

impl Config {
    pub fn from_args(
        iface: Option<String>,
        interval: Option<u64>,
        threshold: Option<u32>,
        no_overwatch: bool,
        quiet: bool,
    ) -> Self {
        let file_cfg = load_config_file();
        Self {
            wifi_iface: iface.unwrap_or_else(|| file_cfg.wifi_iface.clone()),
            scan_interval_secs: interval.unwrap_or(file_cfg.scan_interval_secs),
            match_threshold: threshold.unwrap_or(file_cfg.match_threshold),
            max_breadcrumbs: file_cfg.max_breadcrumbs,
            decay_hours: file_cfg.decay_hours,
            data_dir: file_cfg.data_dir.clone(),
            overwatch_enabled: !no_overwatch && file_cfg.overwatch_enabled,
            evil_twin_detect: file_cfg.evil_twin_detect,
            new_ap_watch: file_cfg.new_ap_watch,
            quiet: quiet || file_cfg.quiet,
        }
    }

    pub fn default_config() -> Self {
        Self {
            wifi_iface: "wlan0".into(),
            scan_interval_secs: 15,
            match_threshold: 30,
            max_breadcrumbs: 2048,
            decay_hours: 8,
            data_dir: DATA_DIR.into(),
            overwatch_enabled: true,
            evil_twin_detect: true,
            new_ap_watch: true,
            quiet: false,
        }
    }

    pub fn config_path() -> PathBuf {
        PathBuf::from(CONFIG_DIR).join("config.env")
    }

    pub fn data_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.data_dir)
    }
}

fn load_config_file() -> Config {
    let mut cfg = Config::default_config();
    let path = Config::config_path();

    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"');
                match key {
                    "TRAIL_IFACE" => cfg.wifi_iface = val.into(),
                    "TRAIL_INTERVAL" => cfg.scan_interval_secs = val.parse().unwrap_or(15),
                    "TRAIL_THRESHOLD" => cfg.match_threshold = val.parse().unwrap_or(30),
                    "TRAIL_MAX_BREADCRUMBS" => cfg.max_breadcrumbs = val.parse().unwrap_or(2048),
                    "TRAIL_DECAY_HOURS" => cfg.decay_hours = val.parse().unwrap_or(8),
                    "TRAIL_DATA_DIR" => cfg.data_dir = val.into(),
                    "TRAIL_OVERWATCH" => cfg.overwatch_enabled = val == "1" || val == "true",
                    "TRAIL_EVIL_TWIN" => cfg.evil_twin_detect = val == "1" || val == "true",
                    "TRAIL_NEW_AP_WATCH" => cfg.new_ap_watch = val == "1" || val == "true",
                    "TRAIL_QUIET" => cfg.quiet = val == "1" || val == "true",
                    _ => {}
                }
            }
        }
    }

    cfg
}

pub fn write_default_config() -> Result<(), String> {
    let dir = Path::new(CONFIG_DIR);
    fs::create_dir_all(dir)
        .map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    let path = Config::config_path();
    let content = r#"# /etc/zeroday/trail/config.env — Trail breadcrumb navigation daemon
#
# Trail uses WiFi fingerprinting to drop breadcrumbs as you walk,
# then guides you back to your exit using signal similarity matching.
#
# Optimized for M5Stack Cardputer Zero (320x170 LCD, 46-key, no mouse)
#
# Usage:
#   trail-ctl start           # start dropping breadcrumbs
#   trail-ctl mark "exit"     # tag a critical waypoint
#   trail-ctl exit            # activate exit guidance
#   trail-ctl pause/resume   # pause/resume dropping
#   trail-ctl stats           # show breadcrumb count and duration
#   trail-ctl dump            # export as GPX/KML
#   trail-ctl clear           # wipe today's breadcrumbs (operational security)

TRAIL_IFACE=wlan0              # WiFi interface for scanning
TRAIL_INTERVAL=15              # seconds between breadcrumb drops
TRAIL_THRESHOLD=30              # minimum similarity % for exit guidance
TRAIL_MAX_BREADCRUMBS=2048     # max breadcrumbs before pruning
TRAIL_DECAY_HOURS=8            # hours before breadcrumbs start decaying
TRAIL_DATA_DIR=/opt/cardputer/trail/breadcrumbs
TRAIL_OVERWATCH=true           # enable radio threat detection
TRAIL_EVIL_TWIN=true           # detect evil twin APs
TRAIL_NEW_AP_WATCH=true        # watch for new APs (baseline learned)
TRAIL_QUIET=false              # suppress non-essential output
"#;
    fs::write(&path, content)
        .map_err(|e| format!("Cannot write {}: {}", path.display(), e))
}