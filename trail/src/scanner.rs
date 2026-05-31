use crate::breadcrumb::{AccessPoint, Breadcrumb};
use std::process::Command;

pub struct GpsData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub fix: bool,
    pub satellites: u32,
    pub speed_mps: f64,
    pub track_deg: f64,
}

pub fn read_gps() -> Option<GpsData> {
    let output = Command::new("gpspipe")
        .args(["-w", "-n", "20"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !line.contains("\"class\":\"TPV\"") { continue; }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
            let mode = data.get("mode").and_then(|m| m.as_u64()).unwrap_or(0);
            if mode < 2 { continue; }
            let lat = data.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let lon = data.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let alt = data.get("altHAE").or_else(|| data.get("alt"))
                .and_then(|v| v.as_f64()).unwrap_or(0.0);
            let speed = data.get("speed").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let track = data.get("track").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if lat == 0.0 && lon == 0.0 { continue; }
            return Some(GpsData {
                lat, lon, alt,
                fix: mode >= 2,
                satellites: 0,
                speed_mps: speed,
                track_deg: track,
            });
        }
    }

    let uart_output = Command::new("cat")
        .args(["/proc/driver/gps"])  // placeholder — fall back to raw NMEA
        .output().ok();
    let _ = uart_output;

    None
}

pub struct WifiScanner {
    iface: String,
}

impl WifiScanner {
    pub fn new(iface: &str) -> Self {
        Self { iface: iface.to_string() }
    }

    pub fn scan(&self) -> Result<Vec<AccessPoint>, String> {
        let aps = self.scan_iw()
            .or_else(|_| self.scan_iwlist())
            .or_else(|_| self.scan_nmcli())?;

        if aps.is_empty() {
            Err("No access points found — interface may be down or scanning unsupported".into())
        } else {
            Ok(aps)
        }
    }

    pub fn current_fingerprint(&self) -> Result<Breadcrumb, String> {
        let aps = self.scan()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(Breadcrumb {
            timestamp,
            fingerprints: aps,
            altitude_hint: None,
            tag: None,
            gps_lat: None,
            gps_lon: None,
            gps_alt: None,
        })
    }

    fn scan_iw(&self) -> Result<Vec<AccessPoint>, String> {
        let output = Command::new("iw")
            .args(["dev", &self.iface, "scan", "trigger"])
            .output()
            .map_err(|e| format!("iw trigger failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Operation not permitted") {
                return Err("Permission denied — try running as root".into());
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(2));

        let output = Command::new("iw")
            .args(["dev", &self.iface, "scan", "dump"])
            .output()
            .map_err(|e| format!("iw dump failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_iw_scan(&stdout)
    }

    fn scan_iwlist(&self) -> Result<Vec<AccessPoint>, String> {
        let output = Command::new("iwlist")
            .args([&self.iface, "scanning"])
            .output()
            .map_err(|e| format!("iwlist failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_iwlist_scan(&stdout)
    }

    fn scan_nmcli(&self) -> Result<Vec<AccessPoint>, String> {
        let output = Command::new("nmcli")
            .args(["-t", "-f", "BSSID,SSID,SIGNAL,FREQ,CHAN", "dev", "wifi", "list"])
            .output()
            .map_err(|e| format!("nmcli failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_nmcli_scan(&stdout)
    }

    pub fn interface_up(&self) -> bool {
        let output = Command::new("ip")
            .args(["link", "show", &self.iface])
            .output()
            .ok();

        output.map(|o| {
            String::from_utf8_lossy(&o.stdout).contains("UP")
        }).unwrap_or(false)
    }

    pub fn connected_bssid(&self) -> Option<String> {
        let output = Command::new("iw")
            .args(["dev", &self.iface, "link"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(bssid) = line.trim().strip_prefix("Connected to ") {
                return Some(bssid.split_whitespace().next().unwrap_or("").to_string());
            }
        }
        None
    }

    pub fn connected_ssid(&self) -> Option<String> {
        let output = Command::new("iw")
            .args(["dev", &self.iface, "link"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(ssid) = line.trim().strip_prefix("SSID:") {
                let ssid = ssid.trim();
                if !ssid.is_empty() {
                    return Some(ssid.to_string());
                }
            }
        }
        None
    }
}

fn parse_iw_scan(stdout: &str) -> Result<Vec<AccessPoint>, String> {
    let mut aps = Vec::new();
    let mut current = None::<AccessPoint>;

    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("BSS ") {
            if let Some(ap) = current.take() {
                aps.push(ap);
            }
            let bssid = line.split_whitespace().nth(1).unwrap_or("").to_string();
            current = Some(AccessPoint {
                bssid,
                ssid: String::new(),
                signal_dbm: -100,
                channel: 0,
                frequency_mhz: 0,
            });
        }

        if let Some(ref mut ap) = current {
            if line.starts_with("SSID:") {
                ap.ssid = line.strip_prefix("SSID:").unwrap_or("").trim().to_string();
            } else if line.starts_with("signal:") {
                let sig = line.strip_prefix("signal:").unwrap_or("").trim();
                ap.signal_dbm = sig.split_whitespace().next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-100);
            } else if line.starts_with("freq:") {
                ap.frequency_mhz = line.strip_prefix("freq:")
                    .unwrap_or("").trim()
                    .split_whitespace().next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.contains("DS Parameter set:") {
                ap.channel = line.split(':').last()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
            }
        }
    }

    if let Some(ap) = current.take() {
        aps.push(ap);
    }

    if aps.is_empty() {
        Err("iw scan returned no APs".into())
    } else {
        Ok(aps)
    }
}

fn parse_iwlist_scan(stdout: &str) -> Result<Vec<AccessPoint>, String> {
    let mut aps = Vec::new();
    let mut current = None::<AccessPoint>;

    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("Cell ") {
            if let Some(ap) = current.take() {
                aps.push(ap);
            }
            let bssid = line.split(':').nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            current = Some(AccessPoint {
                bssid,
                ssid: String::new(),
                signal_dbm: -100,
                channel: 0,
                frequency_mhz: 0,
            });
        }

        if let Some(ref mut ap) = current {
            if line.contains("ESSID:") {
                ap.ssid = line.split(':').nth(1)
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_default();
            } else if line.contains("Signal level=") {
                let sig_part = line.split('=').nth(1).unwrap_or("");
                ap.signal_dbm = sig_part.split_whitespace().next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-100);
            } else if line.contains("Channel:") {
                ap.channel = line.split(':').nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
            } else if line.contains("Frequency:") {
                ap.frequency_mhz = line.split(':').nth(1)
                    .and_then(|s| {
                        s.trim().split_whitespace().next()
                            .and_then(|f| f.parse::<f32>().ok().map(|f| (f * 1000.0) as u32))
                    })
                    .unwrap_or(0);
            }
        }
    }

    if let Some(ap) = current.take() {
        aps.push(ap);
    }

    if aps.is_empty() {
        Err("iwlist scan returned no APs".into())
    } else {
        Ok(aps)
    }
}

fn parse_nmcli_scan(stdout: &str) -> Result<Vec<AccessPoint>, String> {
    let mut aps = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("BSSID") {
            continue;
        }

        let fields: Vec<&str> = line.splitn(5, ':').collect();
        if fields.len() < 5 {
            continue;
        }

        let bssid = fields[0].to_string();
        let ssid = fields[1].to_string();
        let signal: i32 = fields[2].parse().unwrap_or(0);
        let freq: u32 = fields[3].parse().unwrap_or(0);
        let chan: u32 = fields[4].parse().unwrap_or(0);

        let signal_dbm = if signal >= 0 && signal <= 100 {
            -(100 - signal)
        } else {
            signal
        };

        aps.push(AccessPoint {
            bssid,
            ssid,
            signal_dbm,
            channel: chan,
            frequency_mhz: freq,
        });
    }

    if aps.is_empty() {
        Err("nmcli returned no APs".into())
    } else {
        Ok(aps)
    }
}