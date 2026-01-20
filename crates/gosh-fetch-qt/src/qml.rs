use std::pin::Pin;
use std::sync::{Mutex, OnceLock};

use cxx_qt_lib::QString;
use gosh_fetch_core::{
    get_user_agent_presets, init_database, settings_to_engine_config, DownloadsDb, DownloadService,
    EngineCommand, Settings, SettingsDb, TrackerUpdater, UiMessage,
};

#[cxx_qt::bridge]
mod ffi {
    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
    }

    #[cxx_qt::qobject]
    pub struct AppController {}

    impl qobject::AppController {
        #[qinvokable]
        pub fn initialize(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn poll(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn add_download(self: Pin<&mut AppController>, url: QString, options_json: QString);

        #[qinvokable]
        pub fn add_magnet(self: Pin<&mut AppController>, uri: QString, options_json: QString);

        #[qinvokable]
        pub fn add_torrent(self: Pin<&mut AppController>, path: QString, options_json: QString);

        #[qinvokable]
        pub fn pause_download(self: Pin<&mut AppController>, gid: QString);

        #[qinvokable]
        pub fn resume_download(self: Pin<&mut AppController>, gid: QString);

        #[qinvokable]
        pub fn remove_download(self: Pin<&mut AppController>, gid: QString, delete_files: bool);

        #[qinvokable]
        pub fn pause_all(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn resume_all(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn refresh_downloads(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn refresh_stats(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn load_completed(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn clear_completed(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn set_setting(self: Pin<&mut AppController>, key: QString, value: QString);

        #[qinvokable]
        pub fn get_settings_json(self: Pin<&mut AppController>) -> QString;

        #[qinvokable]
        pub fn get_user_agent_presets_json(self: Pin<&mut AppController>) -> QString;

        #[qinvokable]
        pub fn get_user_agent_value(self: Pin<&mut AppController>, index: i32) -> QString;

        #[qinvokable]
        pub fn get_user_agent_index(self: Pin<&mut AppController>, value: QString) -> i32;

        #[qinvokable]
        pub fn update_trackers(self: Pin<&mut AppController>);

        #[qinvokable]
        pub fn open_path(self: Pin<&mut AppController>, path: QString);

        #[qinvokable]
        pub fn get_icon_path(self: Pin<&mut AppController>) -> QString;

        #[qsignal]
        pub fn download_added(self: Pin<&mut AppController>, json: QString);

        #[qsignal]
        pub fn download_updated(self: Pin<&mut AppController>, gid: QString, json: QString);

        #[qsignal]
        pub fn download_removed(self: Pin<&mut AppController>, gid: QString);

        #[qsignal]
        pub fn download_completed(self: Pin<&mut AppController>, json: QString);

        #[qsignal]
        pub fn stats_updated(self: Pin<&mut AppController>, json: QString);

        #[qsignal]
        pub fn downloads_list(self: Pin<&mut AppController>, json: QString);

        #[qsignal]
        pub fn completed_list(self: Pin<&mut AppController>, json: QString);

        #[qsignal]
        pub fn error(self: Pin<&mut AppController>, message: QString);

        #[qsignal]
        pub fn toast(self: Pin<&mut AppController>, message: QString);
    }
}

struct AppState {
    db: gosh_fetch_core::Database,
    settings: Settings,
    cmd_sender: async_channel::Sender<EngineCommand>,
    ui_receiver: async_channel::Receiver<UiMessage>,
}

static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn ensure_state() -> Result<(), String> {
    if APP_STATE.get().is_some() {
        return Ok(());
    }

    let db = init_database().map_err(|e| format!("Failed to initialize database: {}", e))?;
    let settings = SettingsDb::load(&db).unwrap_or_else(|e| {
        log::warn!("Failed to load settings, using defaults: {}", e);
        Settings::default()
    });

    let (ui_sender, ui_receiver) = async_channel::bounded::<UiMessage>(200);
    let (cmd_sender, cmd_receiver) = async_channel::bounded::<EngineCommand>(200);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

    match rt.block_on(DownloadService::new_async(&settings)) {
        Ok(service) => {
            service.spawn(ui_sender, cmd_receiver);
        }
        Err(e) => {
            return Err(format!("Failed to create download service: {}", e));
        }
    }

    restore_incomplete_downloads(&db, &cmd_sender);

    let state = AppState {
        db,
        settings,
        cmd_sender,
        ui_receiver,
    };

    APP_STATE
        .set(Mutex::new(state))
        .map_err(|_| "Failed to initialize app state".to_string())?;

    Ok(())
}

fn restore_incomplete_downloads(db: &gosh_fetch_core::Database, cmd_sender: &async_channel::Sender<EngineCommand>) {
    match DownloadsDb::get_incomplete(db) {
        Ok(incomplete) => {
            if incomplete.is_empty() {
                return;
            }

            log::info!("Restoring {} incomplete downloads", incomplete.len());

            for download in incomplete {
                match download.download_type {
                    gosh_fetch_core::DownloadType::Http => {
                        if let Some(url) = &download.url {
                            let _ = cmd_sender.send_blocking(EngineCommand::AddDownload {
                                url: url.clone(),
                                options: None,
                            });
                        }
                    }
                    gosh_fetch_core::DownloadType::Magnet => {
                        if let Some(uri) = &download.magnet_uri {
                            let _ = cmd_sender.send_blocking(EngineCommand::AddMagnet {
                                uri: uri.clone(),
                                options: None,
                            });
                        }
                    }
                    gosh_fetch_core::DownloadType::Torrent => {
                        log::debug!(
                            "Skipping torrent restoration for {}: engine handles persistence",
                            download.name
                        );
                    }
                    gosh_fetch_core::DownloadType::Ftp => {
                        log::warn!(
                            "Skipping FTP download restoration for {}: not supported",
                            download.name
                        );
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to restore incomplete downloads: {}", e);
        }
    }
}

fn parse_options(options_json: &str) -> Option<gosh_fetch_core::DownloadOptions> {
    let trimmed = options_json.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

fn normalize_path(raw: &str) -> String {
    raw.trim().trim_start_matches("file://").to_string()
}

fn icon_path() -> String {
    if let Ok(path) = std::env::var("GOSH_FETCH_ICON_PATH") {
        if !path.trim().is_empty() {
            return path;
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let candidate = std::path::Path::new(&manifest_dir)
            .join("resources")
            .join("io.github.gosh.Fetch.png");
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent
                .join("..")
                .join("share")
                .join("icons")
                .join("hicolor")
                .join("256x256")
                .join("apps")
                .join("io.github.gosh.Fetch.png");
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }

    "".to_string()
}

impl ffi::qobject::AppController {
    fn with_state<T>(&self, mut f: impl FnMut(&mut AppState) -> T) -> Option<T> {
        if ensure_state().is_err() {
            return None;
        }
        let state_lock = APP_STATE.get()?;
        let mut state = state_lock.lock().ok()?;
        Some(f(&mut state))
    }

    pub fn initialize(self: Pin<&mut Self>) {
        if let Err(e) = ensure_state() {
            self.error(QString::from(&e));
        }
    }

    pub fn poll(self: Pin<&mut Self>) {
        let mut messages: Vec<UiMessage> = Vec::new();
        self.with_state(|state| {
            while let Ok(msg) = state.ui_receiver.try_recv() {
                messages.push(msg);
            }
        });

        for msg in messages {
            match msg {
                UiMessage::EngineReady => {
                    self.toast(QString::from("Download engine ready"));
                }
                UiMessage::DownloadAdded(download) => {
                    self.with_state(|state| {
                        if let Err(e) = DownloadsDb::save(&state.db, &download) {
                            log::error!("Failed to save download: {}", e);
                        }
                    });
                    if let Ok(json) = serde_json::to_string(&download) {
                        self.download_added(QString::from(json));
                    }
                }
                UiMessage::DownloadUpdated(gid, download) => {
                    self.with_state(|state| {
                        if let Err(e) = DownloadsDb::save(&state.db, &download) {
                            log::error!("Failed to save download update: {}", e);
                        }
                    });
                    if let Ok(json) = serde_json::to_string(&download) {
                        self.download_updated(QString::from(gid), QString::from(json));
                    }
                }
                UiMessage::DownloadRemoved(gid) => {
                    self.with_state(|state| {
                        if let Err(e) = DownloadsDb::delete(&state.db, &gid) {
                            log::error!("Failed to delete download record: {}", e);
                        }
                    });
                    self.download_removed(QString::from(gid));
                }
                UiMessage::DownloadCompleted(download) => {
                    self.with_state(|state| {
                        if let Err(e) = DownloadsDb::save(&state.db, &download) {
                            log::error!("Failed to save completed download: {}", e);
                        }
                    });

                    if let Some(settings) = self.with_state(|state| state.settings.clone()) {
                        if settings.enable_notifications {
                            if let Err(e) = notify_rust::Notification::new()
                                .summary("Download Complete")
                                .body(&format!("{} has finished downloading", download.name))
                                .icon("folder-download")
                                .appname("Gosh-Fetch")
                                .show()
                            {
                                log::warn!("Failed to show notification: {}", e);
                            }
                        }
                    }

                    if let Ok(json) = serde_json::to_string(&download) {
                        self.download_completed(QString::from(json));
                    }
                }
                UiMessage::DownloadFailed(_, error) => {
                    self.error(QString::from(error));
                }
                UiMessage::StatsUpdated(stats) => {
                    if let Ok(json) = serde_json::to_string(&stats) {
                        self.stats_updated(QString::from(json));
                    }
                }
                UiMessage::DownloadsList(downloads) => {
                    if let Ok(json) = serde_json::to_string(&downloads) {
                        self.downloads_list(QString::from(json));
                    }
                }
                UiMessage::Error(error) => {
                    self.error(QString::from(error));
                }
            }
        }
    }

    pub fn add_download(self: Pin<&mut Self>, url: QString, options_json: QString) {
        let url = url.to_string();
        if url.trim().is_empty() {
            self.error(QString::from("URL cannot be empty"));
            return;
        }

        let options = parse_options(&options_json.to_string());
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::AddDownload {
                url,
                options,
            });
        });
    }

    pub fn add_magnet(self: Pin<&mut Self>, uri: QString, options_json: QString) {
        let uri = uri.to_string();
        if !uri.starts_with("magnet:") {
            self.error(QString::from("Invalid magnet link"));
            return;
        }

        let options = parse_options(&options_json.to_string());
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::AddMagnet { uri, options });
        });
    }

    pub fn add_torrent(self: Pin<&mut Self>, path: QString, options_json: QString) {
        let path = normalize_path(&path.to_string());
        if path.trim().is_empty() {
            self.error(QString::from("Torrent path is empty"));
            return;
        }

        let data = match std::fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                self.error(QString::from(format!("Failed to read torrent: {}", e)));
                return;
            }
        };

        let options = parse_options(&options_json.to_string());
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::AddTorrent {
                data,
                options,
            });
        });
    }

    pub fn pause_download(self: Pin<&mut Self>, gid: QString) {
        let gid = gid.to_string();
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::Pause(gid));
        });
    }

    pub fn resume_download(self: Pin<&mut Self>, gid: QString) {
        let gid = gid.to_string();
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::Resume(gid));
        });
    }

    pub fn remove_download(self: Pin<&mut Self>, gid: QString, delete_files: bool) {
        let gid = gid.to_string();
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::Remove {
                gid,
                delete_files,
            });
        });
    }

    pub fn pause_all(self: Pin<&mut Self>) {
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::PauseAll);
        });
    }

    pub fn resume_all(self: Pin<&mut Self>) {
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::ResumeAll);
        });
    }

    pub fn refresh_downloads(self: Pin<&mut Self>) {
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::RefreshDownloads);
        });
    }

    pub fn refresh_stats(self: Pin<&mut Self>) {
        let _ = self.with_state(|state| {
            let _ = state.cmd_sender.send_blocking(EngineCommand::RefreshStats);
        });
    }

    pub fn load_completed(self: Pin<&mut Self>) {
        let mut list: Vec<gosh_fetch_core::Download> = Vec::new();
        self.with_state(|state| {
            if let Ok(downloads) = DownloadsDb::get_completed(&state.db, 200) {
                list = downloads;
            }
        });

        if let Ok(json) = serde_json::to_string(&list) {
            self.completed_list(QString::from(json));
        }
    }

    pub fn clear_completed(self: Pin<&mut Self>) {
        self.with_state(|state| {
            if let Err(e) = DownloadsDb::clear_history(&state.db) {
                log::error!("Failed to clear history: {}", e);
            }
        });
    }

    pub fn set_setting(self: Pin<&mut Self>, key: QString, value: QString) {
        let key = key.to_string();
        let value = value.to_string();

        self.with_state(|state| {
            if let Err(e) = SettingsDb::set(&state.db, &key, &value) {
                log::error!("Failed to save setting '{}': {}", key, e);
                return;
            }

            match key.as_str() {
                "download_path" => state.settings.download_path = value.clone(),
                "max_concurrent_downloads" => {
                    state.settings.max_concurrent_downloads = value.parse().unwrap_or(5)
                }
                "max_connections_per_server" => {
                    state.settings.max_connections_per_server = value.parse().unwrap_or(16)
                }
                "split_count" => state.settings.split_count = value.parse().unwrap_or(16),
                "download_speed_limit" => {
                    state.settings.download_speed_limit = value.parse().unwrap_or(0)
                }
                "upload_speed_limit" => {
                    state.settings.upload_speed_limit = value.parse().unwrap_or(0)
                }
                "user_agent" => state.settings.user_agent = value.clone(),
                "enable_notifications" => state.settings.enable_notifications = value == "true",
                "close_to_tray" => state.settings.close_to_tray = value == "true",
                "bt_enable_dht" => state.settings.bt_enable_dht = value == "true",
                "bt_enable_pex" => state.settings.bt_enable_pex = value == "true",
                "bt_enable_lpd" => state.settings.bt_enable_lpd = value == "true",
                "bt_max_peers" => state.settings.bt_max_peers = value.parse().unwrap_or(55),
                "bt_seed_ratio" => state.settings.bt_seed_ratio = value.parse().unwrap_or(1.0),
                "auto_update_trackers" => state.settings.auto_update_trackers = value == "true",
                "delete_files_on_remove" => state.settings.delete_files_on_remove = value == "true",
                "proxy_enabled" => state.settings.proxy_enabled = value == "true",
                "proxy_type" => state.settings.proxy_type = value.clone(),
                "proxy_url" => state.settings.proxy_url = value.clone(),
                "proxy_user" => state.settings.proxy_user = Some(value.clone()).filter(|s| !s.is_empty()),
                "proxy_pass" => state.settings.proxy_pass = Some(value.clone()).filter(|s| !s.is_empty()),
                "min_segment_size" => state.settings.min_segment_size = value.parse().unwrap_or(1024),
                "bt_preallocation" => state.settings.bt_preallocation = value.clone(),
                _ => {}
            }

            let config = settings_to_engine_config(&state.settings);
            let _ = state
                .cmd_sender
                .send_blocking(EngineCommand::UpdateConfig(config));
        });
    }

    pub fn get_settings_json(self: Pin<&mut Self>) -> QString {
        let mut json = String::new();
        self.with_state(|state| {
            if let Ok(value) = serde_json::to_string(&state.settings) {
                json = value;
            }
        });
        QString::from(json)
    }

    pub fn get_user_agent_presets_json(self: Pin<&mut Self>) -> QString {
        let presets = get_user_agent_presets();
        let names: Vec<&str> = presets.iter().map(|(name, _)| *name).collect();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        QString::from(json)
    }

    pub fn get_user_agent_value(self: Pin<&mut Self>, index: i32) -> QString {
        let presets = get_user_agent_presets();
        let idx = index.max(0) as usize;
        let value = presets.get(idx).map(|(_, v)| *v).unwrap_or("");
        QString::from(value)
    }

    pub fn get_user_agent_index(self: Pin<&mut Self>, value: QString) -> i32 {
        let target = value.to_string();
        let presets = get_user_agent_presets();
        for (idx, (_, ua)) in presets.iter().enumerate() {
            if *ua == target {
                return idx as i32;
            }
        }
        0
    }

    pub fn update_trackers(self: Pin<&mut Self>) {
        let db = self.with_state(|state| state.db.clone());
        if db.is_none() {
            return;
        }

        let db = db.unwrap();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("Failed to start runtime for tracker update: {}", e);
                    return;
                }
            };

            let result = rt.block_on(async {
                let mut updater = TrackerUpdater::new();
                let trackers = updater.fetch_trackers().await?;
                gosh_fetch_core::TrackersDb::replace_all(&db, &trackers)?;
                Ok::<usize, gosh_fetch_core::Error>(trackers.len())
            });

            match result {
                Ok(count) => {
                    log::info!("Updated {} trackers", count);
                }
                Err(e) => {
                    log::error!("Failed to update trackers: {}", e);
                }
            }
        });

        self.toast(QString::from("Updating trackers..."));
    }

    pub fn open_path(self: Pin<&mut Self>, path: QString) {
        let path = normalize_path(&path.to_string());
        if path.trim().is_empty() {
            return;
        }
        if let Err(e) = open::that(path) {
            self.error(QString::from(format!("Failed to open path: {}", e)));
        }
    }

    pub fn get_icon_path(self: Pin<&mut Self>) -> QString {
        let path = icon_path();
        if path.is_empty() {
            return QString::from(\"\");
        }
        if path.starts_with(\"file://\") {
            return QString::from(path);
        }
        QString::from(format!(\"file://{}\", path))
    }
}

impl Default for ffi::AppController {
    fn default() -> Self {
        Self {}
    }
}
