//! Database connection management

use crate::error::{Error, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const SCHEMA: &str = include_str!("../../../../migrations/001_initial.sql");

/// Get the database path
pub fn get_db_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("io.github.gosh.Fetch");

    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("gosh-fetch.db")
}

/// Initialize the database with schema
pub fn init_database() -> Result<Database> {
    let path = get_db_path();
    log::info!("Initializing database at: {:?}", path);

    let conn = Connection::open(&path)?;

    // Run migrations
    conn.execute_batch(SCHEMA)?;

    Ok(Database {
        conn: Arc::new(Mutex::new(conn)),
    })
}

/// Database wrapper with thread-safe connection
#[derive(Clone, Debug)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Execute a function with the database connection
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            Error::Database(format!("Failed to lock database: {}", e))
        })?;
        f(&conn).map_err(Into::into)
    }

    /// Execute a function with mutable database connection
    pub fn with_conn_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> rusqlite::Result<T>,
    {
        let mut conn = self.conn.lock().map_err(|e| {
            Error::Database(format!("Failed to lock database: {}", e))
        })?;
        f(&mut conn).map_err(Into::into)
    }
}
