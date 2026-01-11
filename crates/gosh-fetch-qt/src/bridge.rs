//! CXX-Qt bridge module
//!
//! This module defines the Rust/Qt interop using cxx-qt macros.
//! It exposes Rust types and functions to QML.

use gosh_fetch_core::{Download, EngineCommand, GlobalStats, UiMessage};
use once_cell::sync::OnceCell;
use std::pin::Pin;
use std::sync::Mutex;

/// Global command sender - set from main.rs before Qt app starts
static CMD_SENDER: OnceCell<async_channel::Sender<EngineCommand>> = OnceCell::new();

/// Global state for downloads - updated from UI messages
static DOWNLOADS: once_cell::sync::Lazy<Mutex<std::collections::HashMap<String, Download>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(std::collections::HashMap::new()));
static COMPLETED: once_cell::sync::Lazy<Mutex<Vec<Download>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Vec::new()));
static STATS: once_cell::sync::Lazy<Mutex<GlobalStats>> =
    once_cell::sync::Lazy::new(|| Mutex::new(GlobalStats::default()));

/// Set the command sender from main.rs
pub fn set_command_sender(sender: async_channel::Sender<EngineCommand>) {
    let _ = CMD_SENDER.set(sender);
}

/// Handle UI messages from the download engine
pub fn handle_ui_message(msg: UiMessage) {
    match msg {
        UiMessage::DownloadAdded(download) => {
            if let Ok(mut downloads) = DOWNLOADS.lock() {
                downloads.insert(download.gid.clone(), download);
            }
        }
        UiMessage::DownloadUpdated(gid, download) => {
            if let Ok(mut downloads) = DOWNLOADS.lock() {
                downloads.insert(gid, download);
            }
        }
        UiMessage::DownloadRemoved(gid) => {
            if let Ok(mut downloads) = DOWNLOADS.lock() {
                downloads.remove(&gid);
            }
        }
        UiMessage::DownloadCompleted(download) => {
            if let Ok(mut downloads) = DOWNLOADS.lock() {
                downloads.remove(&download.gid);
            }
            if let Ok(mut completed) = COMPLETED.lock() {
                completed.insert(0, download);
            }
        }
        UiMessage::StatsUpdated(stats) => {
            if let Ok(mut s) = STATS.lock() {
                *s = stats;
            }
        }
        UiMessage::DownloadsList(list) => {
            if let Ok(mut downloads) = DOWNLOADS.lock() {
                downloads.clear();
                for d in list {
                    downloads.insert(d.gid.clone(), d);
                }
            }
        }
        UiMessage::Error(err) => {
            log::error!("Engine error: {}", err);
        }
        _ => {}
    }
}

/// Application controller exposed to QML
#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    unsafe extern "RustQt" {
        /// Main application controller for QML
        #[qobject]
        #[qml_element]
        #[qproperty(i32, active_count)]
        #[qproperty(i32, completed_count)]
        #[qproperty(QString, download_speed)]
        #[qproperty(QString, upload_speed)]
        #[qproperty(QString, status_text)]
        type AppController = super::AppControllerRust;

        /// Add a new download
        #[qinvokable]
        fn add_download(self: Pin<&mut AppController>, url: &QString);

        /// Add a torrent file by path
        #[qinvokable]
        fn add_torrent_file(self: Pin<&mut AppController>, path: &QString);

        /// Pause a download
        #[qinvokable]
        fn pause_download(self: Pin<&mut AppController>, gid: &QString);

        /// Resume a download
        #[qinvokable]
        fn resume_download(self: Pin<&mut AppController>, gid: &QString);

        /// Remove a download
        #[qinvokable]
        fn remove_download(self: Pin<&mut AppController>, gid: &QString, delete_files: bool);

        /// Pause all downloads
        #[qinvokable]
        fn pause_all(self: Pin<&mut AppController>);

        /// Resume all downloads
        #[qinvokable]
        fn resume_all(self: Pin<&mut AppController>);

        /// Open folder containing a download
        #[qinvokable]
        fn open_folder(self: &AppController, path: &QString);

        /// Get download info as JSON string
        #[qinvokable]
        fn get_downloads_json(self: &AppController) -> QString;

        /// Get completed downloads as JSON string
        #[qinvokable]
        fn get_completed_json(self: &AppController) -> QString;

        /// Refresh data from global state
        #[qinvokable]
        fn refresh(self: Pin<&mut AppController>);
    }
}

use cxx_qt_lib::QString;
use std::collections::HashMap;

/// Rust implementation of the AppController
pub struct AppControllerRust {
    downloads: HashMap<String, Download>,
    completed: Vec<Download>,
    stats: GlobalStats,

    // Q_PROPERTY backing fields (must be cxx-qt compatible types)
    active_count: i32,
    completed_count: i32,
    download_speed: QString,
    upload_speed: QString,
    status_text: QString,
}

impl Default for AppControllerRust {
    fn default() -> Self {
        Self {
            downloads: HashMap::new(),
            completed: Vec::new(),
            stats: GlobalStats::default(),
            active_count: 0,
            completed_count: 0,
            download_speed: QString::from("0 B/s"),
            upload_speed: QString::from("0 B/s"),
            status_text: QString::from("Ready"),
        }
    }
}

fn send_command(cmd: EngineCommand) {
    if let Some(sender) = CMD_SENDER.get() {
        let _ = sender.send_blocking(cmd);
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.2} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

impl ffi::AppController {
    fn add_download(self: Pin<&mut Self>, url: &QString) {
        let url_str = url.to_string();
        let cmd = if url_str.starts_with("magnet:") {
            EngineCommand::AddMagnet {
                uri: url_str,
                options: None,
            }
        } else {
            EngineCommand::AddDownload {
                url: url_str,
                options: None,
            }
        };
        send_command(cmd);
    }

    fn add_torrent_file(self: Pin<&mut Self>, path: &QString) {
        let path_str = path.to_string();
        if let Ok(data) = std::fs::read(&path_str) {
            send_command(EngineCommand::AddTorrent {
                data,
                options: None,
            });
        } else {
            log::error!("Failed to read torrent file: {}", path_str);
        }
    }

    fn pause_download(self: Pin<&mut Self>, gid: &QString) {
        send_command(EngineCommand::Pause(gid.to_string()));
    }

    fn resume_download(self: Pin<&mut Self>, gid: &QString) {
        send_command(EngineCommand::Resume(gid.to_string()));
    }

    fn remove_download(self: Pin<&mut Self>, gid: &QString, delete_files: bool) {
        send_command(EngineCommand::Remove {
            gid: gid.to_string(),
            delete_files,
        });
    }

    fn pause_all(self: Pin<&mut Self>) {
        send_command(EngineCommand::PauseAll);
    }

    fn resume_all(self: Pin<&mut Self>) {
        send_command(EngineCommand::ResumeAll);
    }

    fn open_folder(&self, path: &QString) {
        let _ = open::that(path.to_string());
    }

    fn get_downloads_json(&self) -> QString {
        // Get downloads from global state
        if let Ok(downloads) = DOWNLOADS.lock() {
            let list: Vec<_> = downloads.values().collect();
            match serde_json::to_string(&list) {
                Ok(json) => return QString::from(&json),
                Err(_) => {}
            }
        }
        QString::from("[]")
    }

    fn get_completed_json(&self) -> QString {
        if let Ok(completed) = COMPLETED.lock() {
            match serde_json::to_string(&*completed) {
                Ok(json) => return QString::from(&json),
                Err(_) => {}
            }
        }
        QString::from("[]")
    }

    fn refresh(mut self: Pin<&mut Self>) {
        // Update from global state
        if let Ok(downloads) = DOWNLOADS.lock() {
            self.as_mut().set_active_count(downloads.len() as i32);
            self.downloads = downloads.clone();
        }
        if let Ok(completed) = COMPLETED.lock() {
            self.as_mut().set_completed_count(completed.len() as i32);
            self.completed = completed.clone();
        }
        if let Ok(stats) = STATS.lock() {
            self.as_mut()
                .set_download_speed(QString::from(&format_speed(stats.download_speed)));
            self.as_mut()
                .set_upload_speed(QString::from(&format_speed(stats.upload_speed)));
            self.stats = stats.clone();
        }

        // Request refresh from engine
        send_command(EngineCommand::RefreshDownloads);
        send_command(EngineCommand::RefreshStats);
    }
}
