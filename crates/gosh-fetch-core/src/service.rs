//! Download service - bridges tokio async runtime with UI main loop

use crate::engine_adapter::EngineAdapter;
use crate::types::{Download, DownloadOptions, GlobalStats, Settings};
use gosh_dl::{DownloadEngine, DownloadEvent, EngineConfig};

/// Commands sent from UI to the engine (via async channel)
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Add an HTTP/HTTPS download
    AddDownload {
        url: String,
        options: Option<DownloadOptions>,
    },
    /// Add a magnet link
    AddMagnet {
        uri: String,
        options: Option<DownloadOptions>,
    },
    /// Add a torrent file
    AddTorrent {
        data: Vec<u8>,
        options: Option<DownloadOptions>,
    },
    /// Pause a download
    Pause(String),
    /// Resume a download
    Resume(String),
    /// Remove a download
    Remove {
        gid: String,
        delete_files: bool,
    },
    /// Pause all downloads
    PauseAll,
    /// Resume all downloads
    ResumeAll,
    /// Update engine configuration
    UpdateConfig(EngineConfig),
    /// Request current downloads list
    RefreshDownloads,
    /// Request global stats
    RefreshStats,
    /// Shutdown the service
    Shutdown,
}

/// Messages sent from engine to UI (via channel)
#[derive(Debug, Clone)]
pub enum UiMessage {
    /// A download was added
    DownloadAdded(Download),
    /// A download was updated
    DownloadUpdated(String, Download),
    /// A download was removed
    DownloadRemoved(String),
    /// A download completed
    DownloadCompleted(Download),
    /// A download failed
    DownloadFailed(String, String),
    /// Global stats updated
    StatsUpdated(GlobalStats),
    /// Full downloads list
    DownloadsList(Vec<Download>),
    /// Error message
    Error(String),
    /// Engine initialized
    EngineReady,
}

/// Download service that runs in a separate thread with tokio
pub struct DownloadService {
    adapter: EngineAdapter,
}

impl DownloadService {
    /// Create a new download service with the given settings
    pub async fn new_async(settings: &Settings) -> Result<Self, gosh_dl::EngineError> {
        let config = settings_to_config(settings);
        let engine = DownloadEngine::new(config).await?;
        let adapter = EngineAdapter::new(engine);

        Ok(Self { adapter })
    }

    /// Get a clone of the engine adapter
    pub fn adapter(&self) -> EngineAdapter {
        self.adapter.clone()
    }

    /// Spawn the service in a background thread
    /// Takes the command receiver to process commands from the UI
    pub fn spawn(
        self,
        ui_sender: async_channel::Sender<UiMessage>,
        cmd_receiver: async_channel::Receiver<EngineCommand>,
    ) {
        let adapter = self.adapter;

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

            rt.block_on(async move {
                // Subscribe to engine events
                let mut event_rx = adapter.engine().subscribe();

                // Notify UI that engine is ready
                let _ = ui_sender.send(UiMessage::EngineReady).await;

                loop {
                    tokio::select! {
                        // Handle commands from UI
                        cmd_result = cmd_receiver.recv() => {
                            match cmd_result {
                                Ok(EngineCommand::Shutdown) => {
                                    log::info!("Download service shutting down");
                                    break;
                                }
                                Ok(cmd) => {
                                    handle_command(&adapter, &ui_sender, cmd).await;
                                }
                                Err(_) => {
                                    log::warn!("Command channel closed");
                                    break;
                                }
                            }
                        }

                        // Handle events from engine
                        event_result = event_rx.recv() => {
                            if let Ok(event) = event_result {
                                handle_engine_event(&adapter, &ui_sender, event).await;
                            }
                        }
                    }
                }
            });
        });
    }
}

/// Handle an event from the engine
async fn handle_engine_event(
    adapter: &EngineAdapter,
    ui_sender: &async_channel::Sender<UiMessage>,
    event: DownloadEvent,
) {
    match event {
        DownloadEvent::Completed { id } => {
            let gid = id.as_uuid().to_string();
            if let Some(download) = adapter.get_status(&gid) {
                log::info!("Download completed: {}", download.name);
                let _ = ui_sender.send(UiMessage::DownloadCompleted(download)).await;
            }
        }
        DownloadEvent::Failed { id, error, .. } => {
            let gid = id.as_uuid().to_string();
            log::error!("Download failed: {} - {}", gid, error);
            let _ = ui_sender.send(UiMessage::DownloadFailed(gid, error)).await;
        }
        DownloadEvent::Progress { id, .. } => {
            let gid = id.as_uuid().to_string();
            if let Some(download) = adapter.get_status(&gid) {
                let _ = ui_sender.send(UiMessage::DownloadUpdated(gid, download)).await;
            }
        }
        DownloadEvent::Removed { .. } => {
            // Handled by the command handler, no need to duplicate
        }
        // Other events can be handled as needed
        _ => {}
    }
}

/// Handle a command from the UI
async fn handle_command(
    adapter: &EngineAdapter,
    ui_sender: &async_channel::Sender<UiMessage>,
    cmd: EngineCommand,
) {
    match cmd {
        EngineCommand::AddDownload { url, options } => {
            match adapter.add_download(url, options).await {
                Ok(gid) => {
                    if let Some(download) = adapter.get_status(&gid) {
                        let _ = ui_sender.send(UiMessage::DownloadAdded(download)).await;
                    }
                }
                Err(e) => {
                    let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
                }
            }
        }

        EngineCommand::AddMagnet { uri, options } => {
            match adapter.add_magnet(&uri, options).await {
                Ok(gid) => {
                    if let Some(download) = adapter.get_status(&gid) {
                        let _ = ui_sender.send(UiMessage::DownloadAdded(download)).await;
                    }
                }
                Err(e) => {
                    let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
                }
            }
        }

        EngineCommand::AddTorrent { data, options } => {
            match adapter.add_torrent(&data, options).await {
                Ok(gid) => {
                    if let Some(download) = adapter.get_status(&gid) {
                        let _ = ui_sender.send(UiMessage::DownloadAdded(download)).await;
                    }
                }
                Err(e) => {
                    let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
                }
            }
        }

        EngineCommand::Pause(gid) => {
            if let Err(e) = adapter.pause(&gid).await {
                let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
            }
        }

        EngineCommand::Resume(gid) => {
            if let Err(e) = adapter.resume(&gid).await {
                let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
            }
        }

        EngineCommand::Remove { gid, delete_files } => {
            if let Err(e) = adapter.remove(&gid, delete_files).await {
                let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
            } else {
                let _ = ui_sender.send(UiMessage::DownloadRemoved(gid)).await;
            }
        }

        EngineCommand::PauseAll => {
            if let Err(e) = adapter.pause_all().await {
                let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
            }
        }

        EngineCommand::ResumeAll => {
            if let Err(e) = adapter.resume_all().await {
                let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
            }
        }

        EngineCommand::UpdateConfig(config) => {
            if let Err(e) = adapter.update_config(config) {
                let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
            }
        }

        EngineCommand::RefreshDownloads => {
            let downloads = adapter.get_all();
            let _ = ui_sender.send(UiMessage::DownloadsList(downloads)).await;
        }

        EngineCommand::RefreshStats => {
            let stats = adapter.get_global_stats();
            let _ = ui_sender.send(UiMessage::StatsUpdated(stats)).await;
        }

        EngineCommand::Shutdown => {
            // Handled in the main loop
        }
    }
}

/// Convert settings to engine configuration
fn settings_to_config(settings: &Settings) -> EngineConfig {
    let download_dir = std::path::PathBuf::from(&settings.download_path);

    // Ensure download directory exists
    if !download_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&download_dir) {
            log::warn!("Failed to create download directory {:?}: {}", download_dir, e);
        }
    }

    // Get database path for session persistence (same location as app database)
    let database_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("io.github.gosh.Fetch")
        .join("engine.db");

    // Ensure engine database directory exists
    if let Some(parent) = database_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    EngineConfig {
        download_dir,
        max_concurrent_downloads: settings.max_concurrent_downloads as usize,
        max_connections_per_download: settings.max_connections_per_server as usize,
        global_download_limit: if settings.download_speed_limit > 0 {
            Some(settings.download_speed_limit)
        } else {
            None
        },
        global_upload_limit: if settings.upload_speed_limit > 0 {
            Some(settings.upload_speed_limit)
        } else {
            None
        },
        user_agent: settings.user_agent.clone(),
        enable_dht: settings.bt_enable_dht,
        enable_pex: settings.bt_enable_pex,
        enable_lpd: settings.bt_enable_lpd,
        max_peers: settings.bt_max_peers as usize,
        seed_ratio: settings.bt_seed_ratio,
        database_path: Some(database_path),
        ..Default::default()
    }
}
