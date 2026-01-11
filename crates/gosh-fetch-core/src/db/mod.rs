//! Database module - SQLite persistence layer

mod connection;
mod downloads;
mod settings;

pub use connection::{get_db_path, init_database, Database};
pub use downloads::DownloadsDb;
pub use settings::{SettingsDb, TrackersDb};
