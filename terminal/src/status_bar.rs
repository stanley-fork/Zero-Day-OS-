use std::fs;
use std::time::{Duration, Instant};

const BATTERY_PATH: &str = "/sys/class/power_supply/bq27220";
const WIFI_PATH: &str = "/sys/class/net/wlan0";
const CPU_TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const UPDATE_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct StatusBar {
    pub battery_pct: String,
    pub battery_status: String,
    pub wifi_ssid: String,
    pub wifi_ip: String,
    pub cpu_temp: String,
    pub load_avg: String,
    pub time: String,
    last_update: Instant,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            battery_pct: "---".into(),
            battery_status: "?".into(),
            wifi_ssid: "---".into(),
            wifi_ip: "---".into(),
            cpu_temp: "--C".into(),
            load_avg: "0.0".into(),
            time: "--:--".into(),
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.battery_pct = read_battery_pct();
        self.battery_status = read_battery_status();
        self.wifi_ip = read_wifi_ip();
        self.cpu_temp = read_cpu_temp();
        self.load_avg = read_load_avg();
        self.time = read_time();
        self.last_update = Instant::now();
    }

    pub fn should_update(&self) -> bool {
        self.last_update.elapsed() >= UPDATE_INTERVAL
    }

    pub fn render(&self, width: u32) -> String {
        let bat = format!("BAT:{}%{}", self.battery_pct, self.battery_status);
        let wifi = format!("W:{}", self.wifi_ip);
        let temp = format!("T:{}", self.cpu_temp);
        let load = format!("L:{}", self.load_avg);
        let time = self.time.clone();

        format!("{} {} {} {} {}", bat, wifi, temp, load, time)
    }
}

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_battery_pct() -> String {
    read_file(&format!("{}/capacity", BATTERY_PATH))
        .unwrap_or_else(|| read_file(&format!("{}/uevent", BATTERY_PATH))
            .and_then(|s| s.lines()
                .find(|l| l.starts_with("POWER_SUPPLY_CAPACITY="))
                .map(|l| l.split('=').nth(1).unwrap_or("0").to_string()))
            .unwrap_or_else(|| "0".to_string()))
}

fn read_battery_status() -> String {
    read_file(&format!("{}/status", BATTERY_PATH))
        .map(|s| {
            match s.as_str() {
                "Charging" => "+",
                "Discharging" => "-",
                "Full" => "=",
                _ => "?",
            }
            .to_string()
        })
        .unwrap_or_else(|| "?".to_string())
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

fn read_cpu_temp() -> String {
    read_file(CPU_TEMP_PATH)
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|t| format!("{:.0}C", t / 1000.0))
        .unwrap_or_else(|| "--C".to_string())
}

fn read_load_avg() -> String {
    read_file("/proc/loadavg")
        .and_then(|s| s.split_whitespace().next().map(|s| s.to_string()))
        .unwrap_or_else(|| "0.0".to_string())
}

fn read_time() -> String {
    let output = std::process::Command::new("date")
        .args(["+%H:%M"])
        .output()
        .ok();
    output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "--:--".to_string())
}