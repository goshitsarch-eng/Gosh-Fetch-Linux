//! Downloads database operations

use crate::db::Database;
use crate::error::Result;
use crate::types::{Download, DownloadState, DownloadType};
use rusqlite::params;

/// Downloads database operations
pub struct DownloadsDb;

impl DownloadsDb {
    /// Save a download to the database
    pub fn save(db: &Database, download: &Download) -> Result<i64> {
        db.with_conn(|conn| {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO downloads
                (gid, name, url, magnet_uri, info_hash, download_type, status,
                 total_size, completed_size, download_speed, upload_speed,
                 save_path, created_at, completed_at, error_message, selected_files)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                "#,
                params![
                    download.gid,
                    download.name,
                    download.url,
                    download.magnet_uri,
                    download.info_hash,
                    download.download_type.to_string(),
                    download.status.to_string(),
                    download.total_size as i64,
                    download.completed_size as i64,
                    download.download_speed as i64,
                    download.upload_speed as i64,
                    download.save_path,
                    download.created_at,
                    download.completed_at,
                    download.error_message,
                    download.selected_files.as_ref().map(|f| {
                        f.iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    }),
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    /// Get a download by GID
    pub fn get_by_gid(db: &Database, gid: &str) -> Result<Option<Download>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, gid, name, url, magnet_uri, info_hash, download_type, status,
                       total_size, completed_size, download_speed, upload_speed,
                       save_path, created_at, completed_at, error_message, selected_files
                FROM downloads WHERE gid = ?1
                "#,
            )?;

            let result = stmt.query_row(params![gid], |row| {
                Ok(row_to_download(row)?)
            });

            match result {
                Ok(download) => Ok(Some(download)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    /// Get all completed downloads (for history)
    pub fn get_completed(db: &Database, limit: i64) -> Result<Vec<Download>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, gid, name, url, magnet_uri, info_hash, download_type, status,
                       total_size, completed_size, download_speed, upload_speed,
                       save_path, created_at, completed_at, error_message, selected_files
                FROM downloads
                WHERE status = 'complete'
                ORDER BY completed_at DESC
                LIMIT ?1
                "#,
            )?;

            let downloads = stmt
                .query_map(params![limit], |row| row_to_download(row))?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(downloads)
        })
    }

    /// Get incomplete downloads (for restoration)
    pub fn get_incomplete(db: &Database) -> Result<Vec<Download>> {
        db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, gid, name, url, magnet_uri, info_hash, download_type, status,
                       total_size, completed_size, download_speed, upload_speed,
                       save_path, created_at, completed_at, error_message, selected_files
                FROM downloads
                WHERE status NOT IN ('complete', 'removed')
                ORDER BY created_at DESC
                "#,
            )?;

            let downloads = stmt
                .query_map([], |row| row_to_download(row))?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(downloads)
        })
    }

    /// Update download status
    pub fn update_status(db: &Database, gid: &str, status: DownloadState) -> Result<()> {
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE downloads SET status = ?1 WHERE gid = ?2",
                params![status.to_string(), gid],
            )?;
            Ok(())
        })
    }

    /// Update completed download
    pub fn mark_completed(db: &Database, gid: &str, completed_at: &str) -> Result<()> {
        db.with_conn(|conn| {
            conn.execute(
                "UPDATE downloads SET status = 'complete', completed_at = ?1 WHERE gid = ?2",
                params![completed_at, gid],
            )?;
            Ok(())
        })
    }

    /// Delete a download record
    pub fn delete(db: &Database, gid: &str) -> Result<()> {
        db.with_conn(|conn| {
            conn.execute("DELETE FROM downloads WHERE gid = ?1", params![gid])?;
            Ok(())
        })
    }

    /// Clear all completed downloads
    pub fn clear_history(db: &Database) -> Result<()> {
        db.with_conn(|conn| {
            conn.execute("DELETE FROM downloads WHERE status = 'complete'", [])?;
            Ok(())
        })
    }

    /// Get count of completed downloads
    pub fn count_completed(db: &Database) -> Result<i64> {
        db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM downloads WHERE status = 'complete'",
                [],
                |row| row.get(0),
            )?;
            Ok(count)
        })
    }
}

fn row_to_download(row: &rusqlite::Row) -> rusqlite::Result<Download> {
    let download_type_str: String = row.get(6)?;
    let status_str: String = row.get(7)?;
    let selected_files_str: Option<String> = row.get(16)?;

    Ok(Download {
        id: row.get(0)?,
        gid: row.get(1)?,
        name: row.get(2)?,
        url: row.get(3)?,
        magnet_uri: row.get(4)?,
        info_hash: row.get(5)?,
        download_type: DownloadType::from(download_type_str.as_str()),
        status: DownloadState::from(status_str.as_str()),
        total_size: row.get::<_, i64>(8)? as u64,
        completed_size: row.get::<_, i64>(9)? as u64,
        download_speed: row.get::<_, i64>(10)? as u64,
        upload_speed: row.get::<_, i64>(11)? as u64,
        save_path: row.get(12)?,
        created_at: row.get(13)?,
        completed_at: row.get(14)?,
        error_message: row.get(15)?,
        connections: 0,
        seeders: 0,
        selected_files: selected_files_str.map(|s| {
            s.split(',')
                .filter_map(|n| n.parse().ok())
                .collect()
        }),
    })
}
