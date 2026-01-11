//! Utility modules for Gosh-Fetch

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};

const TRACKER_LIST_URL: &str =
    "https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_best.txt";

/// Fetches and manages BitTorrent tracker lists
pub struct TrackerUpdater {
    last_update: Option<DateTime<Utc>>,
    trackers: Vec<String>,
}

impl TrackerUpdater {
    pub fn new() -> Self {
        Self {
            last_update: None,
            trackers: Vec::new(),
        }
    }

    pub fn needs_update(&self) -> bool {
        match self.last_update {
            None => true,
            Some(last) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(last);
                duration.num_hours() >= 24
            }
        }
    }

    pub async fn fetch_trackers(&mut self) -> Result<Vec<String>> {
        log::info!("Fetching tracker list from {}", TRACKER_LIST_URL);

        let response = reqwest::get(TRACKER_LIST_URL)
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch trackers: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Failed to fetch trackers: HTTP {}",
                response.status()
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Network(format!("Failed to read response: {}", e)))?;

        let trackers: Vec<String> = text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        log::info!("Fetched {} trackers", trackers.len());

        self.trackers = trackers.clone();
        self.last_update = Some(Utc::now());

        Ok(trackers)
    }

    pub fn get_trackers(&self) -> &[String] {
        &self.trackers
    }

    pub fn set_trackers(&mut self, trackers: Vec<String>) {
        self.trackers = trackers;
        self.last_update = Some(Utc::now());
    }
}

impl Default for TrackerUpdater {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format speed to human-readable string
pub fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return "0 B/s".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Calculate ETA from remaining bytes and speed
pub fn format_eta(remaining: u64, speed: u64) -> String {
    if speed == 0 || remaining == 0 {
        return "--".to_string();
    }

    let seconds = remaining / speed;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

/// Calculate progress percentage
pub fn calculate_progress(completed: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (completed as f64 / total as f64).min(1.0)
}
