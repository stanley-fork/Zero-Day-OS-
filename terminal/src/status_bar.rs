use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BATTERY_PATH: &str = "/sys/class/power_supply/bq27220";
const CPU_TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const UPDATE_INTERVAL: Duration = Duration::from_secs(3);
const BATTERY_LOW_PCT: u32 = 15;
const CPU_TEMP_HIGH: f32 = 75.0;

#[derive(Debug, Clone)]
pub struct StatusBar {
    pub battery_pct: u32,
    pub battery_status: String,
    pub battery_health: String,
    pub wifi_ssid: String,
    pub wifi_ip: String,
    pub wifi_signal: String,
    pub cpu_temp_c: f32,
    pub load_avg_1m: f32,
    pub mem_used_pct: u32,
    pub mem_total_mb: u32,
    pub disk_used_pct: u32,
    pub hostname: String,
    pub uptime_secs: u32,
    pub time: String,
    pub net_ifaces: String,
    pub proc_count: u32,
    last_update: Instant,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            battery_pct: 0,
            battery_status: "?".into(),
            battery_health: "?".into(),
            wifi_ssid: "---".into(),
            wifi_ip: "OFF".into(),
            wifi_signal: String::new(),
            cpu_temp_c: 0.0,
            load_avg_1m: 0.0,
            mem_used_pct: 0,
            mem_total_mb: 0,
            disk_used_pct: 0,
            hostname: "zday".into(),
            uptime_secs: 0,
            time: "--:--".into(),
            net_ifaces: String::new(),
            proc_count: 0,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.battery_pct = read_battery_pct();
        self.battery_status = read_battery_status();
        self.battery_health = read_battery_health();
        let wifi_info = read_wifi_info();
        self.wifi_ssid = wifi_info.0;
        self.wifi_ip = wifi_info.1;
        self.wifi_signal = wifi_info.2;
        self.cpu_temp_c = read_cpu_temp_c();
        self.load_avg_1m = read_load_avg();
        let mem = read_mem_info();
        self.mem_used_pct = mem.0;
        self.mem_total_mb = mem.1;
        self.disk_used_pct = read_disk_pct();
        self.hostname = read_hostname();
        self.uptime_secs = read_uptime_secs();
        self.time = read_time();
        self.net_ifaces = read_active_ifaces();
        self.proc_count = read_proc_count();
        self.last_update = Instant::now();
    }

    pub fn should_update(&self) -> bool {
        self.last_update.elapsed() >= UPDATE_INTERVAL
    }

    pub fn render(&self, width: u32) -> String {
        let wide = self.render_wide();
        if wide.len() <= width as usize {
            wide
        } else {
            self.render_compact()
        }
    }

    fn render_wide(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        let bat_icon = if self.battery_pct <= BATTERY_LOW_PCT {
            "!!"
        } else if self.battery_pct <= 50 {
            "--"
        } else {
            "++"
        };
        parts.push(format!("{}{}%", bat_icon, self.battery_pct));

        match self.battery_status.as_str() {
            "+" => parts.push("CHG".into()),
            "=" => parts.push("FUL".into()),
            _ => {}
        }

        if self.wifi_ip != "OFF" {
            if !self.wifi_ssid.is_empty() && self.wifi_ssid != "---" {
                let ssid_trunc = truncate_str(&self.wifi_ssid, 8);
                parts.push(format!("W:{}", ssid_trunc));
            }
            parts.push(self.wifi_ip.clone());
            if !self.wifi_signal.is_empty() {
                parts.push(format!("({})", self.wifi_signal));
            }
        } else {
            parts.push("W:OFF".into());
        }

        if !self.net_ifaces.is_empty() && self.wifi_ip == "OFF" {
            parts.push(format!("[{}]", self.net_ifaces));
        }

        let temp_icon = if self.cpu_temp_c >= CPU_TEMP_HIGH { "!!" } else { "" };
        parts.push(format!("{}{:.0}C", temp_icon, self.cpu_temp_c));

        parts.push(format!("L:{:.1}", self.load_avg_1m));
        parts.push(format!("M:{}%", self.mem_used_pct));
        parts.push(format!("D:{}%", self.disk_used_pct));

        let uptime_str = format_uptime(self.uptime_secs);
        parts.push(format!("up{}", uptime_str));

        parts.push(self.time.clone());

        parts.join(" ")
    }

    fn render_compact(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        let bat = if self.battery_pct <= BATTERY_LOW_PCT && self.battery_status != "+" {
            format!("{}!!%", self.battery_pct)
        } else {
            format!("{}{}%", self.battery_pct, self.battery_status)
        };
        parts.push(bat);

        if self.wifi_ip != "OFF" {
            parts.push(format!("w{}", self.wifi_ip));
        } else {
            parts.push("w:x".into());
        }

        let temp_warn = if self.cpu_temp_c >= CPU_TEMP_HIGH { "!" } else { "" };
        parts.push(format!("{}{:.0}c", temp_warn, self.cpu_temp_c));
        parts.push(format!("L{:.0}", self.load_avg_1m));
        parts.push(format!("M{}%", self.mem_used_pct));
        parts.push(self.time.clone());

        parts.join(" ")
    }
}

fn format_uptime(secs: u32) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d{}h", secs / 86400, (secs % 86400) / 3600)
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}~", &s[..max_len - 1])
    }
}

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_battery_pct() -> u32 {
    read_file(&format!("{}/capacity", BATTERY_PATH))
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            read_file(&format!("{}/uevent", BATTERY_PATH)).and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("POWER_SUPPLY_CAPACITY="))
                    .and_then(|l| l.split('=').nth(1).and_then(|v| v.parse().ok()))
            })
        })
        .unwrap_or(0)
}

fn read_battery_status() -> String {
    read_file(&format!("{}/status", BATTERY_PATH))
        .map(|s| match s.as_str() {
            "Charging" => "+".to_string(),
            "Discharging" => "-".to_string(),
            "Full" => "=".to_string(),
            "Not charging" => "o".to_string(),
            _ => "?".to_string(),
        })
        .unwrap_or_else(|| "?".to_string())
}

fn read_battery_health() -> String {
    read_file(&format!("{}/health", BATTERY_PATH)).unwrap_or_else(|| "unknown".into())
}

fn read_wifi_info() -> (String, String, String) {
    let ssid = read_wifi_ssid();
    let ip = read_wifi_ip();
    let signal = read_wifi_signal();
    (ssid, ip, signal)
}

fn read_wifi_ssid() -> String {
    let operstate = read_file("/sys/class/net/wlan0/operstate");
    if operstate.as_deref() != Some("up") {
        return "---".into();
    }
    std::process::Command::new("iwgetid")
        .args(["-r"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "---".into())
}

fn read_wifi_signal() -> String {
    read_file("/proc/net/wireless")
        .and_then(|s| {
            s.lines()
                .find(|l| l.contains("wlan0"))
                .and_then(|l| {
                    let fields: Vec<&str> = l.split_whitespace().collect();
                    fields
                        .get(2)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map(|v| format!("{}dBm", v as i32))
                })
        })
        .unwrap_or_default()
}

fn read_wifi_ip() -> String {
    let output = std::process::Command::new("ip")
        .args(["-4", "addr", "show", "wlan0"])
        .output()
        .ok();

    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if let Some(ip) = line.strip_prefix("    inet ") {
                if let Some(addr) = ip.split('/').next() {
                    if addr != "127.0.0.1" {
                        return addr.to_string();
                    }
                }
            }
        }
    }
    "OFF".to_string()
}

fn read_cpu_temp_c() -> f32 {
    read_file(CPU_TEMP_PATH)
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|t| t / 1000.0)
        .unwrap_or(0.0)
}

fn read_load_avg() -> f32 {
    read_file("/proc/loadavg")
        .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse().ok()))
        .unwrap_or(0.0)
}

fn read_mem_info() -> (u32, u32) {
    let content = read_file("/proc/meminfo").unwrap_or_default();
    let mut mem_total: u32 = 0;
    let mut mem_available: u32 = 0;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            mem_available = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }
    let total_mb = mem_total / 1024;
    let used_pct = if mem_total > 0 {
        ((mem_total - mem_available) * 100 / mem_total) as u32
    } else {
        0
    };
    (used_pct, total_mb)
}

fn read_disk_pct() -> u32 {
    std::process::Command::new("df")
        .args(["--output=pcent", "/"])
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .nth(1)
                .and_then(|l| l.trim().trim_end_matches('%').parse().ok())
        })
        .unwrap_or(0)
}

fn read_hostname() -> String {
    read_file("/etc/hostname")
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "zday".into())
}

fn read_uptime_secs() -> u32 {
    read_file("/proc/uptime")
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f32>().ok())
                .map(|secs| secs as u32)
        })
        .unwrap_or(0)
}

fn read_time() -> String {
    let offset_secs = read_tz_offset();
    let secs_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let local_secs = secs_since_epoch as i64 + offset_secs as i64;
    let local_secs = if local_secs < 0 { 0 } else { local_secs as u64 };
    let total_min = (local_secs % 86400) / 60;
    let hrs = total_min / 60;
    let mins = total_min % 60;
    format!("{:02}:{:02}", hrs, mins)
}

fn read_tz_offset() -> i32 {
    read_file("/etc/timezone")
        .and_then(|tz| parse_tz_offset(&tz))
        .or_else(|| {
            read_file("/etc/localtime").and_then(|_| {
                std::process::Command::new("date")
                    .args(["+%z"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        parse_date_offset(&s)
                    })
            })
        })
        .unwrap_or(0)
}

fn parse_tz_offset(tz: &str) -> Option<i32> {
    let tz = tz.trim();
    if tz.starts_with("Etc/GMT") || tz.starts_with("Etc/UTC") {
        return Some(0);
    }
    None
}

fn parse_date_offset(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.len() >= 5 {
        let sign: i32 = if s.starts_with('-') { -1 } else { 1 };
        let digits = s.trim_start_matches('+').trim_start_matches('-');
        let hours: i32 = digits.get(..2)?.parse().ok()?;
        let mins: i32 = digits.get(2..4)?.parse().ok()?;
        Some(sign * (hours * 3600 + mins * 60))
    } else {
        None
    }
}

fn read_active_ifaces() -> String {
    let mut up: Vec<&str> = Vec::new();
    for iface in &["wlan0", "wlan1", "eth0", "usb0"] {
        let path = format!("/sys/class/net/{}/operstate", iface);
        if let Some(state) = read_file(&path) {
            if state == "up" {
                up.push(iface);
            }
        }
    }
    up.join(",")
}

fn read_proc_count() -> u32 {
    fs::read_dir("/proc")
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                        .unwrap_or(false)
                })
                .count() as u32
        })
        .unwrap_or(0)
}