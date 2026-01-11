//! Types module - data structures for Gosh-Fetch
//!
//! These types define the data models used throughout the application.

use serde::{Deserialize, Serialize};

/// Download options for adding new downloads
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadOptions {
    /// Directory to save the file
    pub dir: Option<String>,
    /// Output filename
    pub out: Option<String>,
    /// Number of connections per server
    pub max_connection_per_server: Option<String>,
    /// Custom user agent
    pub user_agent: Option<String>,
    /// Referer URL
    pub referer: Option<String>,
    /// Custom headers
    pub header: Option<Vec<String>>,
    /// File indices to download (for torrents)
    pub select_file: Option<String>,
    /// Seed ratio for torrents
    pub seed_ratio: Option<String>,
    /// Max download speed
    pub max_download_limit: Option<String>,
    /// Max upload speed
    pub max_upload_limit: Option<String>,
}

/// Global download statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalStats {
    pub download_speed: u64,
    pub upload_speed: u64,
    pub num_active: u32,
    pub num_waiting: u32,
    pub num_stopped: u32,
}

/// Torrent file information (for display before adding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub name: String,
    pub info_hash: String,
    pub total_size: u64,
    pub files: Vec<TorrentFileEntry>,
    pub comment: Option<String>,
    pub creation_date: Option<i64>,
    pub announce_list: Vec<String>,
}

/// Single file in a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFileEntry {
    pub index: usize,
    pub path: String,
    pub length: u64,
}

/// Magnet link information (for display before adding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetInfo {
    pub name: Option<String>,
    pub info_hash: String,
    pub trackers: Vec<String>,
}

/// Download model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: i64,
    pub gid: String,
    pub name: String,
    pub url: Option<String>,
    pub magnet_uri: Option<String>,
    pub info_hash: Option<String>,
    pub download_type: DownloadType,
    pub status: DownloadState,
    pub total_size: u64,
    pub completed_size: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub save_path: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub connections: u32,
    pub seeders: u32,
    pub selected_files: Option<Vec<usize>>,
}

impl Default for Download {
    fn default() -> Self {
        Self {
            id: 0,
            gid: String::new(),
            name: String::new(),
            url: None,
            magnet_uri: None,
            info_hash: None,
            download_type: DownloadType::Http,
            status: DownloadState::Waiting,
            total_size: 0,
            completed_size: 0,
            download_speed: 0,
            upload_speed: 0,
            save_path: String::new(),
            created_at: String::new(),
            completed_at: None,
            error_message: None,
            connections: 0,
            seeders: 0,
            selected_files: None,
        }
    }
}

/// Type of download
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DownloadType {
    #[default]
    Http,
    Ftp,
    Torrent,
    Magnet,
}

impl std::fmt::Display for DownloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadType::Http => write!(f, "http"),
            DownloadType::Ftp => write!(f, "ftp"),
            DownloadType::Torrent => write!(f, "torrent"),
            DownloadType::Magnet => write!(f, "magnet"),
        }
    }
}

impl From<&str> for DownloadType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "http" => DownloadType::Http,
            "ftp" => DownloadType::Ftp,
            "torrent" => DownloadType::Torrent,
            "magnet" => DownloadType::Magnet,
            _ => DownloadType::Http,
        }
    }
}

/// Download state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DownloadState {
    Active,
    #[default]
    Waiting,
    Paused,
    Complete,
    Error,
    Removed,
}

impl From<&str> for DownloadState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => DownloadState::Active,
            "waiting" => DownloadState::Waiting,
            "paused" => DownloadState::Paused,
            "complete" => DownloadState::Complete,
            "error" => DownloadState::Error,
            "removed" => DownloadState::Removed,
            _ => DownloadState::Waiting,
        }
    }
}

impl std::fmt::Display for DownloadState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadState::Active => write!(f, "active"),
            DownloadState::Waiting => write!(f, "waiting"),
            DownloadState::Paused => write!(f, "paused"),
            DownloadState::Complete => write!(f, "complete"),
            DownloadState::Error => write!(f, "error"),
            DownloadState::Removed => write!(f, "removed"),
        }
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub download_path: String,
    pub max_concurrent_downloads: u32,
    pub max_connections_per_server: u32,
    pub split_count: u32,
    pub download_speed_limit: u64,
    pub upload_speed_limit: u64,
    pub user_agent: String,
    pub enable_notifications: bool,
    pub close_to_tray: bool,
    pub bt_enable_dht: bool,
    pub bt_enable_pex: bool,
    pub bt_enable_lpd: bool,
    pub bt_max_peers: u32,
    pub bt_seed_ratio: f64,
    pub auto_update_trackers: bool,
    pub delete_files_on_remove: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let download_path = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .to_string_lossy()
            .to_string();

        Self {
            download_path,
            max_concurrent_downloads: 5,
            max_connections_per_server: 16,
            split_count: 16,
            download_speed_limit: 0,
            upload_speed_limit: 0,
            user_agent: "gosh-dl/0.1.0".to_string(),
            enable_notifications: true,
            close_to_tray: true,
            bt_enable_dht: true,
            bt_enable_pex: true,
            bt_enable_lpd: true,
            bt_max_peers: 55,
            bt_seed_ratio: 1.0,
            auto_update_trackers: true,
            delete_files_on_remove: false,
        }
    }
}

/// User agent presets
pub fn get_user_agent_presets() -> Vec<(&'static str, &'static str)> {
    vec![
        ("gosh-dl/0.1.0", "gosh-dl/0.1.0"),
        (
            "Chrome (Windows)",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
        (
            "Chrome (macOS)",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
        (
            "Firefox (Windows)",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
        ),
        (
            "Firefox (Linux)",
            "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0",
        ),
        ("Wget", "Wget/1.21"),
        ("Curl", "curl/8.4.0"),
    ]
}
