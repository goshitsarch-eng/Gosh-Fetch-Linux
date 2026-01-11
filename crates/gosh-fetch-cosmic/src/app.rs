//! COSMIC Application implementation
//!
//! This module implements the cosmic::Application trait to provide
//! the main application logic for the COSMIC desktop frontend.

use cosmic::app::{context_drawer, Core, Task};
use cosmic::iced::Subscription;
use cosmic::widget::nav_bar;
use cosmic::{Application, Element};
use gosh_fetch_core::{
    get_user_agent_presets, init_database, Database, Download, DownloadService, DownloadState,
    DownloadsDb, EngineCommand, GlobalStats, Settings, SettingsDb, UiMessage,
};
use std::collections::HashMap;
use std::time::Duration;

/// Application state
pub struct App {
    core: Core,
    nav: nav_bar::Model,
    page: Page,
    db: Option<Database>,
    settings: Settings,
    cmd_sender: Option<async_channel::Sender<EngineCommand>>,
    downloads: HashMap<String, Download>,
    completed: Vec<Download>,
    stats: GlobalStats,
    context_page: ContextPage,
    // Add download dialog state
    show_add_dialog: bool,
    add_dialog_tab: AddDialogTab,
    url_input: String,
    magnet_input: String,
    torrent_path: Option<String>,
}

/// Add download dialog tab
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AddDialogTab {
    #[default]
    Url,
    Magnet,
    Torrent,
}

/// Current page
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Downloads,
    Completed,
    Settings,
}

/// Context drawer page
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContextPage {
    #[default]
    About,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavSelect(nav_bar::Id),
    ToggleContextPage(ContextPage),

    // Core service messages
    UiMessage(UiMessage),

    // Download actions
    PauseDownload(String),
    ResumeDownload(String),
    RemoveDownload(String, bool),
    PauseAll,
    ResumeAll,

    // Add download dialog
    ShowAddDialog,
    CloseAddDialog,
    AddDialogTabChanged(AddDialogTab),
    UrlInputChanged(String),
    MagnetInputChanged(String),
    BrowseTorrentFile,
    TorrentFileSelected(String),
    SubmitDownload,

    // Completed view
    RemoveFromCompleted(String),
    ClearCompletedHistory,
    OpenDownloadFolder(String),

    // Settings - General
    SettingDownloadPathChanged(String),
    SettingNotificationsChanged(bool),
    SettingCloseToTrayChanged(bool),
    SettingDeleteOnRemoveChanged(bool),
    BrowseDownloadPath,

    // Settings - Connection
    SettingConcurrentDownloadsChanged(u32),
    SettingConnectionsPerServerChanged(u32),
    SettingSplitCountChanged(u32),
    SettingDownloadSpeedLimitChanged(u64),
    SettingUploadSpeedLimitChanged(u64),

    // Settings - User Agent
    SettingUserAgentChanged(usize),

    // Settings - BitTorrent
    SettingAutoUpdateTrackersChanged(bool),
    SettingBtDhtChanged(bool),
    SettingBtPexChanged(bool),
    SettingBtLpdChanged(bool),
    SettingBtMaxPeersChanged(u32),
    SettingBtSeedRatioChanged(f64),

    // Periodic update
    Tick,
}

impl Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.gosh.Fetch.Cosmic";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Initialize navigation
        let mut nav = nav_bar::Model::default();
        nav.insert()
            .text("Downloads")
            .icon(cosmic::widget::icon::from_name("folder-download-symbolic"))
            .data::<Page>(Page::Downloads)
            .activate();
        nav.insert()
            .text("Completed")
            .icon(cosmic::widget::icon::from_name("emblem-ok-symbolic"))
            .data::<Page>(Page::Completed);
        nav.insert()
            .text("Settings")
            .icon(cosmic::widget::icon::from_name("emblem-system-symbolic"))
            .data::<Page>(Page::Settings);

        // Initialize database
        let db = match init_database() {
            Ok(db) => Some(db),
            Err(e) => {
                log::error!("Failed to initialize database: {}", e);
                None
            }
        };

        // Load settings
        let settings = db
            .as_ref()
            .and_then(|db| SettingsDb::load(db).ok())
            .unwrap_or_default();

        // Load completed downloads from database
        let completed = db
            .as_ref()
            .and_then(|db| DownloadsDb::get_completed(db, 100).ok())
            .unwrap_or_default();

        let mut app = Self {
            core,
            nav,
            page: Page::Downloads,
            db,
            settings,
            cmd_sender: None,
            downloads: HashMap::new(),
            completed,
            stats: GlobalStats::default(),
            context_page: ContextPage::About,
            show_add_dialog: false,
            add_dialog_tab: AddDialogTab::Url,
            url_input: String::new(),
            magnet_input: String::new(),
            torrent_path: None,
        };

        // Start download service
        let task = app.start_download_service();

        (app, task)
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        if let Some(page) = self.nav.data::<Page>(id) {
            self.page = *page;
        }
        Task::none()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about_page(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title("About"),
        })
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![]
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        vec![]
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Periodic tick for refreshing download stats
        cosmic::iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::NavSelect(id) => {
                return self.on_nav_select(id);
            }

            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
            }

            Message::UiMessage(msg) => {
                self.handle_ui_message(msg);
            }

            Message::PauseDownload(gid) => {
                self.send_command(EngineCommand::Pause(gid));
            }

            Message::ResumeDownload(gid) => {
                self.send_command(EngineCommand::Resume(gid));
            }

            Message::RemoveDownload(gid, delete_files) => {
                self.send_command(EngineCommand::Remove { gid, delete_files });
            }

            Message::PauseAll => {
                self.send_command(EngineCommand::PauseAll);
            }

            Message::ResumeAll => {
                self.send_command(EngineCommand::ResumeAll);
            }

            // Add download dialog
            Message::ShowAddDialog => {
                self.show_add_dialog = true;
                self.add_dialog_tab = AddDialogTab::Url;
                self.url_input.clear();
                self.magnet_input.clear();
                self.torrent_path = None;
            }

            Message::CloseAddDialog => {
                self.show_add_dialog = false;
            }

            Message::AddDialogTabChanged(tab) => {
                self.add_dialog_tab = tab;
            }

            Message::UrlInputChanged(url) => {
                self.url_input = url;
            }

            Message::MagnetInputChanged(magnet) => {
                self.magnet_input = magnet;
            }

            Message::BrowseTorrentFile => {
                // Use native file dialog via rfd
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Torrent Files", &["torrent"])
                    .pick_file()
                {
                    self.torrent_path = Some(path.to_string_lossy().to_string());
                }
            }

            Message::TorrentFileSelected(path) => {
                self.torrent_path = Some(path);
            }

            Message::SubmitDownload => {
                match self.add_dialog_tab {
                    AddDialogTab::Url => {
                        if !self.url_input.trim().is_empty() {
                            let url = self.url_input.trim().to_string();
                            if url.starts_with("magnet:") {
                                self.send_command(EngineCommand::AddMagnet {
                                    uri: url,
                                    options: None,
                                });
                            } else {
                                self.send_command(EngineCommand::AddDownload {
                                    url,
                                    options: None,
                                });
                            }
                            self.show_add_dialog = false;
                            self.url_input.clear();
                        }
                    }
                    AddDialogTab::Magnet => {
                        if !self.magnet_input.trim().is_empty() {
                            self.send_command(EngineCommand::AddMagnet {
                                uri: self.magnet_input.trim().to_string(),
                                options: None,
                            });
                            self.show_add_dialog = false;
                            self.magnet_input.clear();
                        }
                    }
                    AddDialogTab::Torrent => {
                        if let Some(path) = &self.torrent_path {
                            if let Ok(data) = std::fs::read(path) {
                                self.send_command(EngineCommand::AddTorrent {
                                    data,
                                    options: None,
                                });
                                self.show_add_dialog = false;
                                self.torrent_path = None;
                            }
                        }
                    }
                }
            }

            // Completed view
            Message::RemoveFromCompleted(gid) => {
                self.completed.retain(|d| d.gid != gid);
                if let Some(db) = &self.db {
                    if let Err(e) = DownloadsDb::delete(db, &gid) {
                        log::error!("Failed to delete from database: {}", e);
                    }
                }
            }

            Message::ClearCompletedHistory => {
                self.completed.clear();
                if let Some(db) = &self.db {
                    if let Err(e) = DownloadsDb::clear_history(db) {
                        log::error!("Failed to clear history: {}", e);
                    }
                }
            }

            Message::OpenDownloadFolder(path) => {
                let _ = open::that(&path);
            }

            // Settings - General
            Message::SettingNotificationsChanged(val) => {
                self.settings.enable_notifications = val;
                self.save_settings();
            }

            Message::SettingCloseToTrayChanged(val) => {
                self.settings.close_to_tray = val;
                self.save_settings();
            }

            Message::SettingDeleteOnRemoveChanged(val) => {
                self.settings.delete_files_on_remove = val;
                self.save_settings();
            }

            Message::SettingDownloadPathChanged(path) => {
                self.settings.download_path = path;
                self.save_settings();
            }

            Message::BrowseDownloadPath => {
                // Open native folder picker - for now just log
                // TODO: Use cosmic file_chooser when portal support is stable
                log::info!("Browse download path requested");
            }

            // Settings - Connection
            Message::SettingConcurrentDownloadsChanged(val) => {
                self.settings.max_concurrent_downloads = val;
                self.save_settings();
            }

            Message::SettingConnectionsPerServerChanged(val) => {
                self.settings.max_connections_per_server = val;
                self.save_settings();
            }

            Message::SettingSplitCountChanged(val) => {
                self.settings.split_count = val;
                self.save_settings();
            }

            Message::SettingDownloadSpeedLimitChanged(val) => {
                self.settings.download_speed_limit = val;
                self.save_settings();
            }

            Message::SettingUploadSpeedLimitChanged(val) => {
                self.settings.upload_speed_limit = val;
                self.save_settings();
            }

            // Settings - User Agent
            Message::SettingUserAgentChanged(idx) => {
                let presets = get_user_agent_presets();
                if let Some((_, ua)) = presets.get(idx) {
                    self.settings.user_agent = ua.to_string();
                    self.save_settings();
                }
            }

            // Settings - BitTorrent
            Message::SettingAutoUpdateTrackersChanged(val) => {
                self.settings.auto_update_trackers = val;
                self.save_settings();
            }

            Message::SettingBtDhtChanged(val) => {
                self.settings.bt_enable_dht = val;
                self.save_settings();
            }

            Message::SettingBtPexChanged(val) => {
                self.settings.bt_enable_pex = val;
                self.save_settings();
            }

            Message::SettingBtLpdChanged(val) => {
                self.settings.bt_enable_lpd = val;
                self.save_settings();
            }

            Message::SettingBtMaxPeersChanged(val) => {
                self.settings.bt_max_peers = val;
                self.save_settings();
            }

            Message::SettingBtSeedRatioChanged(val) => {
                self.settings.bt_seed_ratio = val;
                self.save_settings();
            }

            Message::Tick => {
                self.send_command(EngineCommand::RefreshDownloads);
                self.send_command(EngineCommand::RefreshStats);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.page {
            Page::Downloads => self.view_downloads(),
            Page::Completed => self.view_completed(),
            Page::Settings => self.view_settings(),
        }
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        if !self.show_add_dialog {
            return None;
        }

        use cosmic::widget::{button, column, dialog, text, text_input, Container, Row};

        // Simple tab buttons instead of segmented_button to avoid API complexity
        let url_style = if self.add_dialog_tab == AddDialogTab::Url {
            button::suggested("URL")
        } else {
            button::standard("URL")
        };
        let magnet_style = if self.add_dialog_tab == AddDialogTab::Magnet {
            button::suggested("Magnet")
        } else {
            button::standard("Magnet")
        };
        let torrent_style = if self.add_dialog_tab == AddDialogTab::Torrent {
            button::suggested("Torrent")
        } else {
            button::standard("Torrent")
        };

        let tabs = Row::new()
            .push(url_style.on_press(Message::AddDialogTabChanged(AddDialogTab::Url)))
            .push(magnet_style.on_press(Message::AddDialogTabChanged(AddDialogTab::Magnet)))
            .push(torrent_style.on_press(Message::AddDialogTabChanged(AddDialogTab::Torrent)))
            .spacing(8);

        // Tab content based on selected tab
        let tab_content: Element<'_, Message> = match self.add_dialog_tab {
            AddDialogTab::Url => {
                let input = text_input("https://example.com/file.zip", &self.url_input)
                    .on_input(Message::UrlInputChanged)
                    .on_submit(|_| Message::SubmitDownload);

                column::with_capacity(2)
                    .push(input)
                    .push(text::caption("Supports HTTP, HTTPS, FTP, and magnet links"))
                    .spacing(8)
                    .into()
            }
            AddDialogTab::Magnet => {
                let input = text_input("magnet:?xt=urn:btih:...", &self.magnet_input)
                    .on_input(Message::MagnetInputChanged)
                    .on_submit(|_| Message::SubmitDownload);

                column::with_capacity(2)
                    .push(input)
                    .push(text::caption("Paste your magnet link here"))
                    .spacing(8)
                    .into()
            }
            AddDialogTab::Torrent => {
                let path_label = self
                    .torrent_path
                    .as_deref()
                    .unwrap_or("No file selected");

                let browse_row = Row::new()
                    .push(text::body(path_label))
                    .push(cosmic::widget::horizontal_space())
                    .push(button::standard("Browse...").on_press(Message::BrowseTorrentFile))
                    .spacing(8);

                column::with_capacity(2)
                    .push(browse_row)
                    .push(text::caption("Select a .torrent file from your computer"))
                    .spacing(8)
                    .into()
            }
        };

        let content = column::with_capacity(2)
            .push(tabs)
            .push(tab_content)
            .spacing(16);

        let dialog_content = Container::new(content).padding(16);

        Some(
            dialog()
                .title("Add Download")
                .control(dialog_content)
                .primary_action(button::suggested("Add").on_press(Message::SubmitDownload))
                .secondary_action(button::standard("Cancel").on_press(Message::CloseAddDialog))
                .into(),
        )
    }
}

impl App {
    fn start_download_service(&mut self) -> Task<Message> {
        // Create async channels for bidirectional communication
        let (ui_sender, ui_receiver) = async_channel::bounded::<UiMessage>(100);
        let (cmd_sender, cmd_receiver) = async_channel::bounded::<EngineCommand>(100);

        // Store command sender for later use
        self.cmd_sender = Some(cmd_sender);

        // Clone settings for the background thread
        let settings = self.settings.clone();

        // Spawn download service in background thread with tokio runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async {
                match DownloadService::new_async(&settings).await {
                    Ok(service) => {
                        service.spawn(ui_sender.clone(), cmd_receiver);
                        // Keep thread alive
                        loop {
                            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create download service: {}", e);
                        let _ = ui_sender.send(UiMessage::Error(e.to_string())).await;
                    }
                }
            });
        });

        // Create a stream from the async channel receiver wrapped in cosmic::Action::App
        let stream = futures::stream::unfold(ui_receiver, |rx| async move {
            rx.recv()
                .await
                .ok()
                .map(|msg| (cosmic::Action::App(Message::UiMessage(msg)), rx))
        });

        Task::stream(stream)
    }

    fn save_settings(&self) {
        if let Some(db) = &self.db {
            if let Err(e) = SettingsDb::save(db, &self.settings) {
                log::error!("Failed to save settings: {}", e);
            }
        }
    }

    fn send_command(&self, cmd: EngineCommand) {
        if let Some(sender) = &self.cmd_sender {
            let _ = sender.send_blocking(cmd);
        }
    }

    fn handle_ui_message(&mut self, msg: UiMessage) {
        match msg {
            UiMessage::EngineReady => {
                log::info!("Download engine ready");
            }
            UiMessage::DownloadAdded(download) => {
                self.downloads.insert(download.gid.clone(), download);
            }
            UiMessage::DownloadUpdated(gid, download) => {
                self.downloads.insert(gid, download);
            }
            UiMessage::DownloadRemoved(gid) => {
                self.downloads.remove(&gid);
            }
            UiMessage::DownloadCompleted(download) => {
                self.downloads.remove(&download.gid);
                self.completed.insert(0, download);
                self.completed.truncate(100);
            }
            UiMessage::DownloadFailed(gid, error) => {
                log::error!("Download {} failed: {}", gid, error);
            }
            UiMessage::StatsUpdated(stats) => {
                self.stats = stats;
            }
            UiMessage::DownloadsList(downloads) => {
                self.downloads.clear();
                for download in downloads {
                    if download.status != DownloadState::Complete {
                        self.downloads.insert(download.gid.clone(), download);
                    }
                }
            }
            UiMessage::Error(error) => {
                log::error!("Error: {}", error);
            }
        }
    }

    fn view_downloads(&self) -> Element<'_, Message> {
        use cosmic::widget::{button, container, text, Column, Row};

        let mut items: Vec<Element<'_, Message>> = Vec::new();

        // Header with title and stats
        let header = Row::new()
            .push(text::title3("Downloads"))
            .push(cosmic::widget::horizontal_space())
            .push(text::caption(format!(
                "{} active | ↓ {} | ↑ {}",
                self.downloads.len(),
                format_speed(self.stats.download_speed),
                format_speed(self.stats.upload_speed)
            )))
            .spacing(8);
        items.push(header.into());

        // Action buttons
        let actions = Row::new()
            .push(button::suggested("Add Download").on_press(Message::ShowAddDialog))
            .push(button::standard("Pause All").on_press(Message::PauseAll))
            .push(button::standard("Resume All").on_press(Message::ResumeAll))
            .spacing(8);
        items.push(actions.into());

        // Downloads list
        if self.downloads.is_empty() {
            items.push(
                container(text::body(
                    "No active downloads. Click 'Add Download' to get started.",
                ))
                .padding(32)
                .into(),
            );
        } else {
            for download in self.downloads.values() {
                let progress = if download.total_size > 0 {
                    download.completed_size as f32 / download.total_size as f32
                } else {
                    0.0
                };

                let status_icon = match download.status {
                    DownloadState::Active => "media-playback-start-symbolic",
                    DownloadState::Paused => "media-playback-pause-symbolic",
                    DownloadState::Waiting => "content-loading-symbolic",
                    _ => "emblem-default-symbolic",
                };

                let info = Column::new()
                    .push(text::body(&download.name))
                    .push(text::caption(format!(
                        "{:.1}% | {} / {}",
                        progress * 100.0,
                        format_size(download.completed_size),
                        format_size(download.total_size)
                    )));

                let pause_resume = if download.status == DownloadState::Paused {
                    button::icon(cosmic::widget::icon::from_name(
                        "media-playback-start-symbolic",
                    ))
                    .on_press(Message::ResumeDownload(download.gid.clone()))
                } else {
                    button::icon(cosmic::widget::icon::from_name(
                        "media-playback-pause-symbolic",
                    ))
                    .on_press(Message::PauseDownload(download.gid.clone()))
                };

                let row_actions = Row::new()
                    .spacing(4)
                    .push(pause_resume)
                    .push(
                        button::icon(cosmic::widget::icon::from_name("folder-open-symbolic"))
                            .on_press(Message::OpenDownloadFolder(download.save_path.clone())),
                    )
                    .push(
                        button::icon(cosmic::widget::icon::from_name("user-trash-symbolic"))
                            .on_press(Message::RemoveDownload(download.gid.clone(), false)),
                    );

                let download_row = Row::new()
                    .spacing(8)
                    .push(cosmic::widget::icon::from_name(status_icon).size(24))
                    .push(info)
                    .push(cosmic::widget::horizontal_space())
                    .push(row_actions);

                items.push(download_row.into());
            }
        }

        let content = Column::with_children(items).spacing(8).padding(16);

        container(cosmic::widget::scrollable(content))
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    fn view_completed(&self) -> Element<'_, Message> {
        use cosmic::widget::{button, container, text, Column, Row};

        let mut items: Vec<Element<'_, Message>> = Vec::new();

        // Header with title and clear button
        let header = Row::new()
            .push(text::title3("Completed Downloads"))
            .push(cosmic::widget::horizontal_space())
            .push(text::caption(format!("{} downloads", self.completed.len())))
            .push(
                button::icon(cosmic::widget::icon::from_name("user-trash-symbolic"))
                    .on_press(Message::ClearCompletedHistory),
            )
            .spacing(8);
        items.push(header.into());

        if self.completed.is_empty() {
            items.push(
                container(text::body("No completed downloads yet."))
                    .padding(32)
                    .into(),
            );
        } else {
            for download in &self.completed {
                let info = Column::new()
                    .push(text::body(&download.name))
                    .push(text::caption(format!(
                        "{} | {}",
                        format_size(download.total_size),
                        &download.save_path
                    )));

                let row_actions = Row::new()
                    .spacing(4)
                    .push(
                        button::icon(cosmic::widget::icon::from_name("folder-open-symbolic"))
                            .on_press(Message::OpenDownloadFolder(download.save_path.clone())),
                    )
                    .push(
                        button::icon(cosmic::widget::icon::from_name("user-trash-symbolic"))
                            .on_press(Message::RemoveFromCompleted(download.gid.clone())),
                    );

                let download_row = Row::new()
                    .spacing(8)
                    .push(cosmic::widget::icon::from_name("emblem-ok-symbolic").size(24))
                    .push(info)
                    .push(cosmic::widget::horizontal_space())
                    .push(row_actions);

                items.push(download_row.into());
            }
        }

        let content = Column::with_children(items).spacing(8).padding(16);

        container(cosmic::widget::scrollable(content))
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        use cosmic::widget::{button, container, settings, slider, text, toggler, Column, Row};

        let mut content = Column::new().spacing(16).padding(16);

        content = content.push(text::title3("Settings"));

        // General section
        let general_section = settings::section()
            .title("General")
            .add(settings::item(
                "Download Location",
                Row::new()
                    .push(text::body(&self.settings.download_path))
                    .push(button::standard("Browse").on_press(Message::BrowseDownloadPath))
                    .spacing(8),
            ))
            .add(settings::item(
                "Enable Notifications",
                toggler(self.settings.enable_notifications)
                    .on_toggle(Message::SettingNotificationsChanged),
            ))
            .add(settings::item(
                "Close to System Tray",
                toggler(self.settings.close_to_tray).on_toggle(Message::SettingCloseToTrayChanged),
            ))
            .add(settings::item(
                "Delete Files When Removing",
                toggler(self.settings.delete_files_on_remove)
                    .on_toggle(Message::SettingDeleteOnRemoveChanged),
            ));

        content = content.push(general_section);

        // Connection section - using sliders for numeric values
        let dl_speed_mb = self.settings.download_speed_limit as f32 / 1_048_576.0;
        let ul_speed_mb = self.settings.upload_speed_limit as f32 / 1_048_576.0;

        let dl_speed_label = if self.settings.download_speed_limit == 0 {
            "Unlimited".to_string()
        } else {
            format!("{:.0} MB/s", dl_speed_mb)
        };

        let ul_speed_label = if self.settings.upload_speed_limit == 0 {
            "Unlimited".to_string()
        } else {
            format!("{:.0} MB/s", ul_speed_mb)
        };

        let connection_section = settings::section()
            .title("Connection")
            .add(settings::item(
                "Max Concurrent Downloads",
                Row::new()
                    .push(
                        slider(1.0..=20.0, self.settings.max_concurrent_downloads as f32, |v| {
                            Message::SettingConcurrentDownloadsChanged(v as u32)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(format!("{}", self.settings.max_concurrent_downloads)))
                    .spacing(8),
            ))
            .add(settings::item(
                "Connections per Server",
                Row::new()
                    .push(
                        slider(1.0..=32.0, self.settings.max_connections_per_server as f32, |v| {
                            Message::SettingConnectionsPerServerChanged(v as u32)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(format!("{}", self.settings.max_connections_per_server)))
                    .spacing(8),
            ))
            .add(settings::item(
                "Split Count",
                Row::new()
                    .push(
                        slider(1.0..=64.0, self.settings.split_count as f32, |v| {
                            Message::SettingSplitCountChanged(v as u32)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(format!("{}", self.settings.split_count)))
                    .spacing(8),
            ))
            .add(settings::item(
                "Download Speed Limit",
                Row::new()
                    .push(
                        slider(0.0..=100.0, dl_speed_mb, |v| {
                            Message::SettingDownloadSpeedLimitChanged((v * 1_048_576.0) as u64)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(dl_speed_label))
                    .spacing(8),
            ))
            .add(settings::item(
                "Upload Speed Limit",
                Row::new()
                    .push(
                        slider(0.0..=100.0, ul_speed_mb, |v| {
                            Message::SettingUploadSpeedLimitChanged((v * 1_048_576.0) as u64)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(ul_speed_label))
                    .spacing(8),
            ));

        content = content.push(connection_section);

        // User Agent section - display current selection with cycle button
        let ua_presets = get_user_agent_presets();
        let current_ua_name = ua_presets
            .iter()
            .find(|(_, ua)| *ua == self.settings.user_agent)
            .map(|(name, _)| *name)
            .unwrap_or("Custom");

        let current_ua_idx = ua_presets
            .iter()
            .position(|(_, ua)| *ua == self.settings.user_agent)
            .unwrap_or(0);
        let next_idx = (current_ua_idx + 1) % ua_presets.len();

        let ua_section = settings::section()
            .title("User Agent")
            .add(settings::item(
                "Identify as",
                Row::new()
                    .push(text::body(current_ua_name))
                    .push(button::standard("Change").on_press(Message::SettingUserAgentChanged(next_idx)))
                    .spacing(8),
            ));

        content = content.push(ua_section);

        // BitTorrent section
        let seed_ratio_label = format!("{:.1}", self.settings.bt_seed_ratio);

        let bittorrent_section = settings::section()
            .title("BitTorrent")
            .add(settings::item(
                "Enable DHT",
                toggler(self.settings.bt_enable_dht).on_toggle(Message::SettingBtDhtChanged),
            ))
            .add(settings::item(
                "Enable PEX",
                toggler(self.settings.bt_enable_pex).on_toggle(Message::SettingBtPexChanged),
            ))
            .add(settings::item(
                "Enable LPD",
                toggler(self.settings.bt_enable_lpd).on_toggle(Message::SettingBtLpdChanged),
            ))
            .add(settings::item(
                "Max Peers",
                Row::new()
                    .push(
                        slider(10.0..=200.0, self.settings.bt_max_peers as f32, |v| {
                            Message::SettingBtMaxPeersChanged(v as u32)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(format!("{}", self.settings.bt_max_peers)))
                    .spacing(8),
            ))
            .add(settings::item(
                "Seed Ratio",
                Row::new()
                    .push(
                        slider(0.0..=5.0, self.settings.bt_seed_ratio as f32, |v| {
                            Message::SettingBtSeedRatioChanged(v as f64)
                        })
                        .width(cosmic::iced::Length::Fixed(150.0)),
                    )
                    .push(text::body(seed_ratio_label))
                    .spacing(8),
            ))
            .add(settings::item(
                "Auto-Update Tracker List",
                toggler(self.settings.auto_update_trackers)
                    .on_toggle(Message::SettingAutoUpdateTrackersChanged),
            ));

        content = content.push(bittorrent_section);

        container(cosmic::widget::scrollable(content))
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    fn about_page(&self) -> Element<'_, Message> {
        use cosmic::widget::{text, Column};

        Column::new()
            .push(text::title3("Gosh-Fetch"))
            .push(text::body("Version 2.0.0"))
            .push(text::body(
                "A modern download manager with native Rust engine",
            ))
            .push(text::body(""))
            .push(text::body("Features:"))
            .push(text::body("- HTTP/HTTPS segmented downloads"))
            .push(text::body("- BitTorrent and Magnet support"))
            .push(text::body("- DHT, PEX, LPD peer discovery"))
            .spacing(8)
            .padding(16)
            .into()
    }
}

// Helper functions for formatting
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_size(bytes_per_sec))
}
