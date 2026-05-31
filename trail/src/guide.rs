use crate::breadcrumb::{Breadcrumb, ExitDirection, ExitGuidance, ThreatLevel};

const EVIL_TWIN_MIN_SIGNAL: i32 = -70;

pub struct OverwatchDetector {
    baseline_bssids: Vec<String>,
    baseline_ssid_count: std::collections::HashMap<String, usize>,
    last_known_ssid: Option<String>,
    last_known_bssid: Option<String>,
}

impl OverwatchDetector {
    pub fn new() -> Self {
        Self {
            baseline_bssids: Vec::new(),
            baseline_ssid_count: std::collections::HashMap::new(),
            last_known_ssid: None,
            last_known_bssid: None,
        }
    }

    pub fn learn_baseline(&mut self, bcs: &[Breadcrumb]) {
        self.baseline_bssids.clear();
        self.baseline_ssid_count.clear();
        for bc in bcs {
            for ap in &bc.fingerprints {
                if !self.baseline_bssids.contains(&ap.bssid) {
                    self.baseline_bssids.push(ap.bssid.clone());
                }
                *self.baseline_ssid_count.entry(ap.ssid.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn detect_evil_twin(&self, current: &Breadcrumb) -> Option<ThreatLevel> {
        let connected_ssid = self.last_known_ssid.as_deref().unwrap_or("");
        if connected_ssid.is_empty() { return None; }

        let matches: Vec<&crate::breadcrumb::AccessPoint> = current.fingerprints.iter()
            .filter(|ap| ap.ssid == connected_ssid && ap.signal_dbm >= EVIL_TWIN_MIN_SIGNAL)
            .collect();

        if matches.len() > 1 {
            let known_bssid = self.last_known_bssid.as_deref().unwrap_or("");
            let has_unknown = matches.iter().any(|ap| ap.bssid != known_bssid);
            if has_unknown { return Some(ThreatLevel::Crit); }
        } else if matches.len() == 1 {
            let known_bssid = self.last_known_bssid.as_deref().unwrap_or("");
            if matches[0].bssid != known_bssid { return Some(ThreatLevel::Warn); }
        }
        None
    }

    pub fn detect_new_aps(&self, current: &Breadcrumb) -> ThreatLevel {
        if self.baseline_bssids.is_empty() { return ThreatLevel::None; }
        let new_count = current.fingerprints.iter()
            .filter(|ap| !self.baseline_bssids.contains(&ap.bssid))
            .count();
        if new_count > 3 { ThreatLevel::Watch }
        else { ThreatLevel::None }
    }

    pub fn set_connected(&mut self, ssid: Option<String>, bssid: Option<String>) {
        self.last_known_ssid = ssid;
        self.last_known_bssid = bssid;
    }
}

pub struct GuideEngine {
    threshold: u32,
}

impl GuideEngine {
    pub fn new(threshold: u32) -> Self {
        Self { threshold }
    }

    pub fn navigate(
        &self,
        current: &Breadcrumb,
        trail: &[Breadcrumb],
        now_epoch: u64,
    ) -> ExitGuidance {
        if trail.is_empty() {
            return ExitGuidance {
                direction: ExitDirection::Lost,
                match_pct: 0,
                waypoint_time: 0,
                waypoint_tag: None,
                distance_hint: "No breadcrumbs".into(),
            };
        }

        let matches = crate::breadcrumb::find_exit_path(current, trail, now_epoch);
        if matches.is_empty() {
            return ExitGuidance {
                direction: ExitDirection::Lost,
                match_pct: 0,
                waypoint_time: 0,
                waypoint_tag: None,
                distance_hint: "No path found".into(),
            };
        }

        let best = &matches[0];
        if best.match_pct < self.threshold {
            return ExitGuidance {
                direction: ExitDirection::Lost,
                match_pct: best.match_pct,
                waypoint_time: best.waypoint_time,
                waypoint_tag: best.waypoint_tag.clone(),
                distance_hint: format!("Weak match ({}%)", best.match_pct),
            };
        }

        best.clone()
    }

    pub fn format_guidance(guidance: &ExitGuidance, width: u16) -> String {
        let tag = guidance.waypoint_tag.as_deref().unwrap_or("waypoint");
        match guidance.direction {
            ExitDirection::Here => {
                format!("✓ EXIT FOUND — {}% match [{}]", guidance.match_pct, tag)
            }
            ExitDirection::Lost => {
                format!("!! NO PATH — {} {}", guidance.match_pct, guidance.distance_hint)
            }
            _ => {
                let dir = format!("{}", guidance.direction);
                let pct = format!("{}%", guidance.match_pct);
                let now_epoch = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let elapsed = now_epoch.saturating_sub(guidance.waypoint_time);
                let time_str = if elapsed < 60 { format!("{}s ago", elapsed) }
                    else if elapsed < 3600 { format!("{}m ago", elapsed / 60) }
                    else { format!("{}h ago", elapsed / 3600) };
                let line = format!("{} {} [{}] {}", dir, pct, tag, time_str);
                if line.len() > width as usize {
                    format!("{} {} [{}]", dir, pct, tag)
                } else {
                    line
                }
            }
        }
    }
}