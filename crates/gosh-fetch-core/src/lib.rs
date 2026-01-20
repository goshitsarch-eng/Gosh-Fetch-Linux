//! Gosh-Fetch Core Library
//!
//! This crate provides the core business logic, download engine adapter,
//! database operations, and service layer for the Gosh-Fetch download manager.
//! It is UI-agnostic and can be used with any frontend (Qt, COSMIC, CLI, etc.)

pub mod db;
pub mod engine_adapter;
pub mod error;
pub mod service;
pub mod types;
pub mod utils;

// Re-exports for convenience
pub use db::{get_db_path, init_database, Database, DownloadsDb, SettingsDb, TrackersDb};
pub use engine_adapter::{EngineAdapter, PeerInfo, TorrentFileInfo};
pub use error::{Error, Result};
pub use service::{settings_to_engine_config, DownloadService, EngineCommand, UiMessage};
pub use types::*;
pub use utils::{calculate_progress, format_bytes, format_eta, format_speed, TrackerUpdater};

// Re-export gosh-dl types that frontends might need
pub use gosh_dl::EngineConfig;
