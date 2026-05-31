use crate::breadcrumb::Breadcrumb;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

const DEFAULT_DATA_DIR: &str = "/opt/cardputer/trail/breadcrumbs";

pub struct BreadcrumbStore {
    data_dir: PathBuf,
}

impl BreadcrumbStore {
    pub fn new() -> Self {
        Self { data_dir: PathBuf::from(DEFAULT_DATA_DIR) }
    }

    pub fn with_dir(dir: &str) -> Self {
        Self { data_dir: PathBuf::from(dir) }
    }

    pub fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Cannot create {}: {}", self.data_dir.display(), e))
    }

    pub fn today_filename(&self) -> PathBuf {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let days = secs / 86400;
        let filename = format!("{:010}.jsonl", days);
        self.data_dir.join(filename)
    }

    pub fn append(&self, bc: &Breadcrumb) -> Result<(), String> {
        self.ensure_dir()?;
        let path = self.today_filename();
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
        let json = serde_json::to_string(bc)
            .map_err(|e| format!("Serialize error: {}", e))?;
        writeln!(file, "{}", json)
            .map_err(|e| format!("Write error: {}", e))
    }

    pub fn load_today(&self) -> Result<Vec<Breadcrumb>, String> {
        let path = self.today_filename();
        self.load_file(&path)
    }

    pub fn load_recent(&self, count: usize) -> Result<Vec<Breadcrumb>, String> {
        self.ensure_dir()?;
        let mut all = self.load_all()?;
        all.reverse();
        all.truncate(count);
        Ok(all)
    }

    pub fn load_all(&self) -> Result<Vec<Breadcrumb>, String> {
        self.ensure_dir()?;
        let mut all = Vec::new();
        let entries = fs::read_dir(&self.data_dir)
            .map_err(|e| format!("Cannot read {}: {}", self.data_dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                let mut bcs = self.load_file(&path)?;
                all.append(&mut bcs);
            }
        }

        all.sort_by_key(|b| b.timestamp);
        Ok(all)
    }

    fn load_file(&self, path: &Path) -> Result<Vec<Breadcrumb>, String> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(path)
            .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
        let reader = std::io::BufReader::new(file);
        let mut bcs = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Read error: {}", e))?;
            let line = line.trim();
            if line.is_empty() { continue; }
            match serde_json::from_str::<Breadcrumb>(line) {
                Ok(bc) => bcs.push(bc),
                Err(e) => log::warn!("Skipping malformed breadcrumb: {}", e),
            }
        }
        Ok(bcs)
    }

    pub fn clear_today(&self) -> Result<(), String> {
        let path = self.today_filename();
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Cannot remove {}: {}", path.display(), e))
        } else {
            Ok(())
        }
    }

    pub fn clear_all(&self) -> Result<(), String> {
        self.ensure_dir()?;
        let entries = fs::read_dir(&self.data_dir)
            .map_err(|e| format!("Cannot read {}: {}", self.data_dir.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                fs::remove_file(&path)
                    .map_err(|e| format!("Cannot remove {}: {}", path.display(), e))?;
            }
        }
        Ok(())
    }

    pub fn export_gpx(&self, output_path: &str) -> Result<(), String> {
        let bcs = self.load_all()?;
        if bcs.is_empty() {
            return Err("No breadcrumbs to export".into());
        }
        let mut file = fs::File::create(output_path)
            .map_err(|e| format!("Cannot create {}: {}", output_path, e))?;

        macro_rules! wl {
            ($($arg:tt)*) => { writeln!(file, $($arg)*).map_err(|e| format!("Write error: {}", e))? }
        }

        wl!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        wl!("<gpx version=\"1.1\" creator=\"zeroday-trail\">");
        wl!("  <trk>");
        wl!("    <name>Trail breadcrumbs</name>");
        wl!("    <trkseg>");

        for bc in &bcs {
            let time = format_epoch(bc.timestamp);
            let tag = bc.tag.as_deref().unwrap_or("waypoint");
            wl!("      <trkpt lat=\"0\" lon=\"0\">");
            wl!("        <time>{}</time>", time);
            wl!("        <name>{}</name>", tag);
            wl!("        <desc>{} APs seen</desc>", bc.fingerprints.len());
            wl!("      </trkpt>");
        }

        wl!("    </trkseg>");
        wl!("  </trk>");
        wl!("</gpx>");
        Ok(())
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

fn format_epoch(secs: u64) -> String {
    let days = secs / 86400;
    let rem = secs % 86400;
    let hrs = rem / 3600;
    let mins = (rem % 3600) / 60;
    let base_days = 719528;
    format!("Day {} {:02}:{:02}Z", days + base_days, hrs, mins)
}

use std::io;

impl io::Write for BreadcrumbStore {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}