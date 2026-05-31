use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_BREADCRUMBS: usize = 2048;
const DECAY_HOURS: u64 = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub timestamp: u64,
    pub fingerprints: Vec<AccessPoint>,
    pub altitude_hint: Option<i32>,
    pub tag: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
    pub gps_alt: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPoint {
    pub bssid: String,
    pub ssid: String,
    pub signal_dbm: i32,
    pub channel: u32,
    pub frequency_mhz: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ThreatLevel {
    None,
    Watch,
    Warn,
    Crit,
}

impl Default for ThreatLevel {
    fn default() -> Self {
        ThreatLevel::None
    }
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatLevel::None => write!(f, "OK"),
            ThreatLevel::Watch => write!(f, "WATCH"),
            ThreatLevel::Warn => write!(f, "WARN"),
            ThreatLevel::Crit => write!(f, "CRIT"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitGuidance {
    pub direction: ExitDirection,
    pub match_pct: u32,
    pub waypoint_time: u64,
    pub waypoint_tag: Option<String>,
    pub distance_hint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExitDirection {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
    Here,
    Lost,
}

impl std::fmt::Display for ExitDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ExitDirection::Forward => "↑ FWD",
            ExitDirection::Back => "← BACK",
            ExitDirection::Left => "← LEFT",
            ExitDirection::Right => "→ RIGHT",
            ExitDirection::Up => "↑ UP",
            ExitDirection::Down => "↓ DOWN",
            ExitDirection::Here => "✓ HERE",
            ExitDirection::Lost => "!! LOST",
        };
        write!(f, "{}", s)
    }
}

pub fn similarity(a: &Breadcrumb, b: &Breadcrumb, now_epoch: u64) -> u32 {
    let a_map: HashMap<&str, i32> = a.fingerprints.iter()
        .map(|ap| (ap.bssid.as_str(), ap.signal_dbm))
        .collect();
    let b_map: HashMap<&str, i32> = b.fingerprints.iter()
        .map(|ap| (ap.bssid.as_str(), ap.signal_dbm))
        .collect();

    let shared: Vec<(&str, i32, i32)> = a_map.keys()
        .filter_map(|k| {
            let sig_a = *a_map.get(k)?;
            let sig_b = *b_map.get(k)?;
            Some((*k, sig_a, sig_b))
        })
        .collect();

    if shared.is_empty() {
        return 0;
    }

    let mut total_weight: f64 = 0.0;
    let mut total_score: f64 = 0.0;
    for (_bssid, sig_a, sig_b) in &shared {
        let weight = ((sig_a + 100).max(1) as f64 / 100.0)
                   * ((sig_b + 100).max(1) as f64 / 100.0);
        let diff = (sig_a - sig_b).abs() as f64;
        let score = (1.0 / (1.0 + diff / 10.0)).max(0.0);
        total_weight += weight;
        total_score += weight * score;
    }

    let shared_ratio = (shared.len() as f64)
        / (a_map.len().max(b_map.len()) as f64).max(1.0);
    let base_score = total_score / total_weight.max(0.001);

    let age_a = now_epoch.saturating_sub(a.timestamp);
    let age_b = now_epoch.saturating_sub(b.timestamp);
    let max_age = age_a.max(age_b);
    let decay = if max_age < DECAY_HOURS * 3600 { 1.0 }
                else { 0.5_f64.powf((max_age as f64 / (DECAY_HOURS * 3600) as f64) - 1.0) };

    let tag_bonus = match (&a.tag, &b.tag) {
        (Some(ta), Some(tb)) if ta == tb => 0.1,
        (Some(_), None) | (None, Some(_)) => 0.05,
        _ => 0.0,
    };

    ((base_score * shared_ratio * decay + tag_bonus) * 100.0).min(100.0) as u32
}

pub fn find_exit_path(
    current: &Breadcrumb,
    trail: &[Breadcrumb],
    now_epoch: u64,
) -> Vec<ExitGuidance> {
    let tagged_exits: Vec<&Breadcrumb> = trail.iter()
        .filter(|b| b.tag.as_deref() == Some("exit") || b.tag.as_deref() == Some("entrance"))
        .collect();

    let targets: Vec<&Breadcrumb> = if !tagged_exits.is_empty() {
        tagged_exits
    } else {
        let earliest = trail.first();
        match earliest {
            Some(bc) => vec![bc],
            None => return vec![ExitGuidance {
                direction: ExitDirection::Lost,
                match_pct: 0,
                waypoint_time: 0,
                waypoint_tag: None,
                distance_hint: "No breadcrumbs".into(),
            }],
        }
    };

    let mut results: Vec<ExitGuidance> = targets.iter()
        .filter_map(|target| {
            let pct = similarity(current, target, now_epoch);
            if pct < 5 { return None; }

            let direction = infer_direction(current, target);
            let elapsed = now_epoch.saturating_sub(current.timestamp);
            let dist_hint = format_elapsed(elapsed);

            Some(ExitGuidance {
                direction,
                match_pct: pct,
                waypoint_time: target.timestamp,
                waypoint_tag: target.tag.clone(),
                distance_hint: dist_hint,
            })
        })
        .collect();

    results.sort_by(|a, b| b.match_pct.cmp(&a.match_pct));
    results.truncate(3);
    results
}

fn infer_direction(current: &Breadcrumb, target: &Breadcrumb) -> ExitDirection {
    let current_avg: f64 = if current.fingerprints.is_empty() { -50.0 }
        else { current.fingerprints.iter().map(|ap| ap.signal_dbm as f64).sum::<f64>() / current.fingerprints.len() as f64 };
    let target_avg: f64 = if target.fingerprints.is_empty() { -50.0 }
        else { target.fingerprints.iter().map(|ap| ap.signal_dbm as f64).sum::<f64>() / target.fingerprints.len() as f64 };

    let shared_current_in_target: Vec<&AccessPoint> = current.fingerprints.iter()
        .filter(|ap| target.fingerprints.iter().any(|t| t.bssid == ap.bssid))
        .collect();

    let signal_change: f64 = shared_current_in_target.iter()
        .map(|ap| {
            if let Some(target_ap) = target.fingerprints.iter().find(|t| t.bssid == ap.bssid) {
                (target_ap.signal_dbm - ap.signal_dbm) as f64
            } else { 0.0 }
        })
        .sum();

    let unique_current = current.fingerprints.len() as f64
        - shared_current_in_target.len() as f64;
    let unique_target = target.fingerprints.len() as f64
        - target.fingerprints.iter()
            .filter(|ap| current.fingerprints.iter().any(|c| c.bssid == ap.bssid))
            .count() as f64;

    if current.fingerprints.is_empty() || target.fingerprints.is_empty() {
        return ExitDirection::Lost;
    }

    let signal_strength_ratio = current_avg / target_avg;
    let signal_delta = signal_change / shared_current_in_target.len().max(1) as f64;

    if signal_strength_ratio > 1.1 && signal_delta > 3.0 {
        ExitDirection::Forward
    } else if signal_strength_ratio < 0.9 && signal_delta < -3.0 {
        ExitDirection::Back
    } else if unique_current > unique_target + 2.0 {
        ExitDirection::Back
    } else if unique_target > unique_current + 2.0 {
        ExitDirection::Forward
    } else if signal_delta.abs() < 2.0 && signal_strength_ratio > 0.95 {
        ExitDirection::Here
    } else if signal_delta > 0.0 {
        ExitDirection::Forward
    } else {
        ExitDirection::Back
    }
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 { format!("{}s ago", secs) }
    else if secs < 3600 { format!("{}m ago", secs / 60) }
    else { format!("{}h{}m ago", secs / 3600, (secs % 3600) / 60) }
}

pub struct BreadcrumbJournal {
    breadcrumbs: Vec<Breadcrumb>,
    max_size: usize,
}

impl BreadcrumbJournal {
    pub fn new() -> Self {
        Self {
            breadcrumbs: Vec::new(),
            max_size: MAX_BREADCRUMBS,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            breadcrumbs: Vec::new(),
            max_size: cap,
        }
    }

    pub fn drop(&mut self, bc: Breadcrumb) {
        if self.breadcrumbs.len() >= self.max_size {
            let oldest_idx = self.breadcrumbs.iter()
                .enumerate()
                .filter(|(_, b)| b.tag.is_none())
                .min_by_key(|(_, b)| b.timestamp)
                .map(|(i, _)| i);
            if let Some(idx) = oldest_idx {
                self.breadcrumbs.remove(idx);
            } else {
                self.breadcrumbs.remove(0);
            }
        }
        self.breadcrumbs.push(bc);
    }

    pub fn mark(&mut self, timestamp: u64, tag: String) -> bool {
        if let Some(bc) = self.breadcrumbs.iter_mut().find(|b| b.timestamp == timestamp) {
            bc.tag = Some(tag);
            true
        } else if let Some(bc) = self.breadcrumbs.last_mut() {
            bc.tag = Some(tag);
            true
        } else {
            false
        }
    }

    pub fn trail(&self) -> &[Breadcrumb] {
        &self.breadcrumbs
    }

    pub fn last(&self) -> Option<&Breadcrumb> {
        self.breadcrumbs.last()
    }

    pub fn tagged_exits(&self) -> Vec<&Breadcrumb> {
        self.breadcrumbs.iter()
            .filter(|b| b.tag.as_deref() == Some("exit") || b.tag.as_deref() == Some("entrance"))
            .collect()
    }

    pub fn prune_older_than(&mut self, epoch: u64) {
        self.breadcrumbs.retain(|b| b.timestamp >= epoch);
    }

    pub fn clear(&mut self) {
        self.breadcrumbs.clear();
    }

    pub fn len(&self) -> usize {
        self.breadcrumbs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.breadcrumbs.is_empty()
    }
}