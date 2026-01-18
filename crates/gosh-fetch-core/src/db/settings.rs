//! Settings database operations

use crate::db::Database;
use crate::error::Result;
use crate::types::Settings;
use rusqlite::params;

/// Settings database operations
pub struct SettingsDb;

impl SettingsDb {
    /// Load all settings from database
    pub fn load(db: &Database) -> Result<Settings> {
        let mut settings = Settings::default();

        db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            for row in rows {
                let (key, value) = row?;
                match key.as_str() {
                    "download_path" => {
                        // Expand ~ to home directory
                        settings.download_path = if value.starts_with("~/") {
                            if let Some(home) = dirs::home_dir() {
                                home.join(&value[2..]).to_string_lossy().to_string()
                            } else {
                                value
                            }
                        } else if value == "~" {
                            dirs::home_dir()
                                .map(|h| h.to_string_lossy().to_string())
                                .unwrap_or(value)
                        } else {
                            value
                        };
                    }
                    "max_concurrent_downloads" => {
                        settings.max_concurrent_downloads = value.parse().unwrap_or(5);
                    }
                    "max_connections_per_server" => {
                        settings.max_connections_per_server = value.parse().unwrap_or(16);
                    }
                    "split_count" => {
                        settings.split_count = value.parse().unwrap_or(16);
                    }
                    "download_speed_limit" => {
                        settings.download_speed_limit = value.parse().unwrap_or(0);
                    }
                    "upload_speed_limit" => {
                        settings.upload_speed_limit = value.parse().unwrap_or(0);
                    }
                    "user_agent" => settings.user_agent = value,
                    "enable_notifications" => {
                        settings.enable_notifications = value == "true";
                    }
                    "close_to_tray" => {
                        settings.close_to_tray = value == "true";
                    }
                    "bt_enable_dht" => {
                        settings.bt_enable_dht = value == "true";
                    }
                    "bt_enable_pex" => {
                        settings.bt_enable_pex = value == "true";
                    }
                    "bt_enable_lpd" => {
                        settings.bt_enable_lpd = value == "true";
                    }
                    "bt_max_peers" => {
                        settings.bt_max_peers = value.parse().unwrap_or(55);
                    }
                    "bt_seed_ratio" => {
                        settings.bt_seed_ratio = value.parse().unwrap_or(1.0);
                    }
                    "auto_update_trackers" => {
                        settings.auto_update_trackers = value == "true";
                    }
                    "delete_files_on_remove" => {
                        settings.delete_files_on_remove = value == "true";
                    }
                    "proxy_enabled" => {
                        settings.proxy_enabled = value == "true";
                    }
                    "proxy_type" => settings.proxy_type = value,
                    "proxy_url" => settings.proxy_url = value,
                    "proxy_user" => settings.proxy_user = Some(value).filter(|s| !s.is_empty()),
                    "proxy_pass" => settings.proxy_pass = Some(value).filter(|s| !s.is_empty()),
                    "min_segment_size" => {
                        settings.min_segment_size = value.parse().unwrap_or(1024);
                    }
                    "bt_preallocation" => settings.bt_preallocation = value,
                    _ => {}
                }
            }

            Ok(())
        })?;

        Ok(settings)
    }

    /// Save a single setting
    pub fn set(db: &Database, key: &str, value: &str) -> Result<()> {
        db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
                params![key, value],
            )?;
            Ok(())
        })
    }

    /// Save all settings
    pub fn save(db: &Database, settings: &Settings) -> Result<()> {
        Self::set(db, "download_path", &settings.download_path)?;
        Self::set(db, "max_concurrent_downloads", &settings.max_concurrent_downloads.to_string())?;
        Self::set(db, "max_connections_per_server", &settings.max_connections_per_server.to_string())?;
        Self::set(db, "split_count", &settings.split_count.to_string())?;
        Self::set(db, "download_speed_limit", &settings.download_speed_limit.to_string())?;
        Self::set(db, "upload_speed_limit", &settings.upload_speed_limit.to_string())?;
        Self::set(db, "user_agent", &settings.user_agent)?;
        Self::set(db, "enable_notifications", if settings.enable_notifications { "true" } else { "false" })?;
        Self::set(db, "close_to_tray", if settings.close_to_tray { "true" } else { "false" })?;
        Self::set(db, "bt_enable_dht", if settings.bt_enable_dht { "true" } else { "false" })?;
        Self::set(db, "bt_enable_pex", if settings.bt_enable_pex { "true" } else { "false" })?;
        Self::set(db, "bt_enable_lpd", if settings.bt_enable_lpd { "true" } else { "false" })?;
        Self::set(db, "bt_max_peers", &settings.bt_max_peers.to_string())?;
        Self::set(db, "bt_seed_ratio", &settings.bt_seed_ratio.to_string())?;
        Self::set(db, "auto_update_trackers", if settings.auto_update_trackers { "true" } else { "false" })?;
        Self::set(db, "delete_files_on_remove", if settings.delete_files_on_remove { "true" } else { "false" })?;
        Self::set(db, "proxy_enabled", if settings.proxy_enabled { "true" } else { "false" })?;
        Self::set(db, "proxy_type", &settings.proxy_type)?;
        Self::set(db, "proxy_url", &settings.proxy_url)?;
        if let Some(ref user) = settings.proxy_user {
            Self::set(db, "proxy_user", user)?;
        }
        if let Some(ref pass) = settings.proxy_pass {
            Self::set(db, "proxy_pass", pass)?;
        }
        Self::set(db, "min_segment_size", &settings.min_segment_size.to_string())?;
        Self::set(db, "bt_preallocation", &settings.bt_preallocation)?;
        Ok(())
    }

    /// Get a single setting value
    pub fn get(db: &Database, key: &str) -> Result<Option<String>> {
        db.with_conn(|conn| {
            let result = conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            );

            match result {
                Ok(value) => Ok(Some(value)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }
}

/// Tracker database operations
pub struct TrackersDb;

impl TrackersDb {
    /// Get all enabled trackers
    pub fn get_enabled(db: &Database) -> Result<Vec<String>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT url FROM trackers WHERE enabled = 1")?;
            let trackers = stmt
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            Ok(trackers)
        })
    }

    /// Replace all trackers
    pub fn replace_all(db: &Database, trackers: &[String]) -> Result<()> {
        db.with_conn_mut(|conn| {
            let tx = conn.transaction()?;

            tx.execute("DELETE FROM trackers", [])?;

            for tracker in trackers {
                tx.execute(
                    "INSERT INTO trackers (url, enabled) VALUES (?1, 1)",
                    params![tracker],
                )?;
            }

            tx.execute(
                "UPDATE tracker_meta SET last_updated = CURRENT_TIMESTAMP WHERE id = 1",
                [],
            )?;

            tx.commit()?;
            Ok(())
        })
    }

    /// Get last update time
    pub fn get_last_updated(db: &Database) -> Result<Option<String>> {
        db.with_conn(|conn| {
            let result = conn.query_row(
                "SELECT last_updated FROM tracker_meta WHERE id = 1",
                [],
                |row| row.get(0),
            );

            match result {
                Ok(value) => Ok(value),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }
}
