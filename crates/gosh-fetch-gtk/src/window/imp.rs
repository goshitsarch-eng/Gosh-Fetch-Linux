//! Window implementation

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{OnceCell, RefCell};

use crate::dialogs::AddDownloadDialog;
use crate::views::{CompletedView, DownloadsView, SettingsView};
use gosh_fetch_core::{
    format_speed, Database, Download, DownloadState, DownloadType, DownloadsDb, EngineCommand,
    GlobalStats,
};

#[derive(Default)]
pub struct GoshFetchWindow {
    pub db: OnceCell<Database>,
    pub cmd_sender: OnceCell<async_channel::Sender<EngineCommand>>,
    pub toast_overlay: OnceCell<adw::ToastOverlay>,
    pub downloads_list: RefCell<Vec<Download>>,
    pub completed_list: RefCell<Vec<Download>>,
    pub stats: RefCell<GlobalStats>,

    // UI components
    pub nav_view: OnceCell<adw::NavigationView>,
    pub downloads_view: OnceCell<DownloadsView>,
    pub completed_view: OnceCell<CompletedView>,
    pub settings_view: OnceCell<SettingsView>,

    // Sidebar labels
    pub downloads_badge: OnceCell<gtk::Label>,
    pub completed_badge: OnceCell<gtk::Label>,
    pub speed_label: OnceCell<gtk::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for GoshFetchWindow {
    const NAME: &'static str = "GoshFetchWindow";
    type Type = super::GoshFetchWindow;
    type ParentType = adw::ApplicationWindow;
}

impl ObjectImpl for GoshFetchWindow {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for GoshFetchWindow {}
impl WindowImpl for GoshFetchWindow {}
impl ApplicationWindowImpl for GoshFetchWindow {}
impl AdwApplicationWindowImpl for GoshFetchWindow {}

impl GoshFetchWindow {
    pub fn setup_ui(&self, window: &super::GoshFetchWindow) {
        // Create toast overlay
        let toast_overlay = adw::ToastOverlay::new();
        let _ = self.toast_overlay.set(toast_overlay.clone());

        // Create navigation split view
        let split_view = adw::NavigationSplitView::new();
        split_view.set_min_sidebar_width(240.0);
        split_view.set_max_sidebar_width(300.0);

        // Create navigation view for content FIRST (before sidebar, as sidebar needs it)
        let nav_view = adw::NavigationView::new();
        let _ = self.nav_view.set(nav_view.clone());

        // Create sidebar (needs nav_view to be set)
        let sidebar = self.create_sidebar(window);
        split_view.set_sidebar(Some(&sidebar));

        // Create views
        let downloads_view = DownloadsView::new(window);
        let completed_view = CompletedView::new(window);
        let settings_view = SettingsView::new(window);

        let _ = self.downloads_view.set(downloads_view.clone());
        let _ = self.completed_view.set(completed_view.clone());
        let _ = self.settings_view.set(settings_view.clone());

        // Add pages to navigation view
        let downloads_page = adw::NavigationPage::builder()
            .title("Downloads")
            .tag("downloads")
            .child(&downloads_view)
            .build();

        let completed_page = adw::NavigationPage::builder()
            .title("Completed")
            .tag("completed")
            .child(&completed_view)
            .build();

        let settings_page = adw::NavigationPage::builder()
            .title("Settings")
            .tag("settings")
            .child(&settings_view)
            .build();

        nav_view.add(&downloads_page);
        nav_view.add(&completed_page);
        nav_view.add(&settings_page);

        // Wrap navigation view in a page for the content
        let content_page = adw::NavigationPage::builder()
            .title("Content")
            .child(&nav_view)
            .build();
        split_view.set_content(Some(&content_page));

        // Wrap in toast overlay
        toast_overlay.set_child(Some(&split_view));

        // Set window content
        window.set_content(Some(&toast_overlay));

        // Load completed downloads from database
        self.load_completed_downloads();

        // Restore incomplete downloads
        self.restore_incomplete_downloads();
    }

    fn load_completed_downloads(&self) {
        if let Some(db) = self.db.get() {
            match DownloadsDb::get_completed(db, 100) {
                Ok(downloads) => {
                    // Store in memory
                    *self.completed_list.borrow_mut() = downloads.clone();

                    // Update view
                    if let Some(view) = self.completed_view.get() {
                        view.set_downloads(&downloads);
                    }

                    self.update_badges();
                    log::info!(
                        "Loaded {} completed downloads from database",
                        downloads.len()
                    );
                }
                Err(e) => {
                    log::error!("Failed to load completed downloads: {}", e);
                }
            }
        }
    }

    fn restore_incomplete_downloads(&self) {
        if let Some(db) = self.db.get() {
            match DownloadsDb::get_incomplete(db) {
                Ok(incomplete) => {
                    if incomplete.is_empty() {
                        return;
                    }

                    log::info!("Restoring {} incomplete downloads", incomplete.len());

                    for download in incomplete {
                        match download.download_type {
                            DownloadType::Http => {
                                // Restore HTTP download using URL
                                if let Some(url) = &download.url {
                                    if let Some(sender) = self.cmd_sender.get() {
                                        let _ = sender.send_blocking(EngineCommand::AddDownload {
                                            url: url.clone(),
                                            options: None,
                                        });
                                    }
                                }
                            }
                            DownloadType::Magnet => {
                                // Restore magnet download using URI
                                if let Some(uri) = &download.magnet_uri {
                                    if let Some(sender) = self.cmd_sender.get() {
                                        let _ = sender.send_blocking(EngineCommand::AddMagnet {
                                            uri: uri.clone(),
                                            options: None,
                                        });
                                    }
                                }
                            }
                            DownloadType::Torrent => {
                                // Torrent files aren't stored in DB, so we can't restore them
                                // The engine's own persistence should handle active torrents
                                log::debug!(
                                    "Skipping torrent restoration for {}: engine handles persistence",
                                    download.name
                                );
                            }
                            DownloadType::Ftp => {
                                // FTP is not supported by the engine
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
    }

    fn create_sidebar(&self, window: &super::GoshFetchWindow) -> adw::NavigationPage {
        let sidebar_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header
        let header = adw::HeaderBar::new();
        header.set_show_title(false);
        sidebar_box.append(&header);

        // App title
        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        title_box.set_margin_start(16);
        title_box.set_margin_end(16);
        title_box.set_margin_top(8);
        title_box.set_margin_bottom(16);

        let icon = gtk::Image::from_icon_name("folder-download-symbolic");
        icon.set_pixel_size(32);
        icon.add_css_class("accent");

        let title = gtk::Label::new(Some("Gosh-Fetch"));
        title.add_css_class("title-2");

        title_box.append(&icon);
        title_box.append(&title);
        sidebar_box.append(&title_box);

        // Navigation list
        let nav_list = gtk::ListBox::new();
        nav_list.set_selection_mode(gtk::SelectionMode::Single);
        nav_list.add_css_class("navigation-sidebar");

        // Downloads row
        let downloads_row = self.create_nav_row("Downloads", "folder-download-symbolic");
        let downloads_badge = gtk::Label::new(Some("0"));
        downloads_badge.add_css_class("badge");
        downloads_badge.set_visible(false);
        downloads_row.add_suffix(&downloads_badge);
        let _ = self.downloads_badge.set(downloads_badge);
        nav_list.append(&downloads_row);

        // Completed row
        let completed_row = self.create_nav_row("Completed", "emblem-ok-symbolic");
        let completed_badge = gtk::Label::new(Some("0"));
        completed_badge.add_css_class("badge");
        completed_badge.set_visible(false);
        completed_row.add_suffix(&completed_badge);
        let _ = self.completed_badge.set(completed_badge);
        nav_list.append(&completed_row);

        // Settings row
        let settings_row = self.create_nav_row("Settings", "emblem-system-symbolic");
        nav_list.append(&settings_row);

        // About row
        let about_row = self.create_nav_row("About", "help-about-symbolic");
        nav_list.append(&about_row);

        // Handle row selection
        let nav_view = self.nav_view.get().unwrap().clone();
        let window_weak = window.downgrade();
        nav_list.connect_row_activated(move |_, row| {
            let index = row.index();
            match index {
                0 => nav_view.replace_with_tags(&["downloads"]),
                1 => nav_view.replace_with_tags(&["completed"]),
                2 => nav_view.replace_with_tags(&["settings"]),
                3 => {
                    if let Some(window) = window_weak.upgrade() {
                        if let Some(app) = window.application() {
                            app.activate_action("about", None);
                        }
                    }
                }
                _ => {}
            }
        });

        // Select downloads by default
        if let Some(first_row) = nav_list.row_at_index(0) {
            nav_list.select_row(Some(&first_row));
        }

        sidebar_box.append(&nav_list);

        // Spacer
        let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        sidebar_box.append(&spacer);

        // Speed display at bottom
        let speed_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        speed_box.set_margin_start(16);
        speed_box.set_margin_end(16);
        speed_box.set_margin_top(8);
        speed_box.set_margin_bottom(16);
        speed_box.set_halign(gtk::Align::Center);

        let speed_label = gtk::Label::new(Some("↓ 0 B/s  ↑ 0 B/s"));
        speed_label.add_css_class("dim-label");
        let _ = self.speed_label.set(speed_label.clone());

        speed_box.append(&speed_label);
        sidebar_box.append(&speed_box);

        adw::NavigationPage::builder()
            .title("Navigation")
            .child(&sidebar_box)
            .build()
    }

    fn create_nav_row(&self, title: &str, icon_name: &str) -> adw::ActionRow {
        let row = adw::ActionRow::new();
        row.set_title(title);

        let icon = gtk::Image::from_icon_name(icon_name);
        row.add_prefix(&icon);

        row.set_activatable(true);
        row
    }

    pub fn add_download(&self, download: &Download) {
        let mut downloads = self.downloads_list.borrow_mut();
        if !downloads.iter().any(|d| d.gid == download.gid) {
            downloads.push(download.clone());
        }
        drop(downloads);

        if let Some(view) = self.downloads_view.get() {
            view.add_download(download);
        }

        self.update_badges();
    }

    pub fn update_download(&self, gid: &str, download: &Download) {
        let mut downloads = self.downloads_list.borrow_mut();
        if let Some(existing) = downloads.iter_mut().find(|d| d.gid == gid) {
            *existing = download.clone();
        }
        drop(downloads);

        if let Some(view) = self.downloads_view.get() {
            view.update_download(gid, download);
        }

        self.update_badges();
    }

    pub fn remove_download(&self, gid: &str) {
        let mut downloads = self.downloads_list.borrow_mut();
        downloads.retain(|d| d.gid != gid);
        drop(downloads);

        if let Some(view) = self.downloads_view.get() {
            view.remove_download(gid);
        }

        self.update_badges();
    }

    pub fn set_downloads(&self, downloads: Vec<Download>) {
        *self.downloads_list.borrow_mut() = downloads.clone();

        if let Some(view) = self.downloads_view.get() {
            view.set_downloads(&downloads);
        }

        self.update_badges();
    }

    pub fn add_to_completed(&self, download: &Download) {
        // Save to database
        if let Some(db) = self.db.get() {
            if let Err(e) = DownloadsDb::save(db, download) {
                log::error!("Failed to save completed download to database: {}", e);
            }
        }

        let mut completed = self.completed_list.borrow_mut();
        if !completed.iter().any(|d| d.gid == download.gid) {
            completed.insert(0, download.clone());
            // Limit to 100 items
            completed.truncate(100);
        }
        drop(completed);

        if let Some(view) = self.completed_view.get() {
            view.add_download(download);
        }

        self.update_badges();
    }

    pub fn update_stats(&self, stats: &GlobalStats) {
        *self.stats.borrow_mut() = stats.clone();

        if let Some(label) = self.speed_label.get() {
            let dl = format_speed(stats.download_speed);
            let ul = format_speed(stats.upload_speed);
            label.set_text(&format!("↓ {}  ↑ {}", dl, ul));
        }

        self.update_badges();
    }

    fn update_badges(&self) {
        let downloads = self.downloads_list.borrow();
        let active_count = downloads
            .iter()
            .filter(|d| matches!(d.status, DownloadState::Active | DownloadState::Waiting))
            .count();

        if let Some(badge) = self.downloads_badge.get() {
            if active_count > 0 {
                badge.set_text(&active_count.to_string());
                badge.set_visible(true);
            } else {
                badge.set_visible(false);
            }
        }

        let completed = self.completed_list.borrow();
        let completed_count = completed.len();

        if let Some(badge) = self.completed_badge.get() {
            if completed_count > 0 {
                badge.set_text(&completed_count.to_string());
                badge.set_visible(true);
            } else {
                badge.set_visible(false);
            }
        }
    }

    pub fn show_add_download_dialog(&self, window: &super::GoshFetchWindow) {
        let dialog = AddDownloadDialog::new(window);
        dialog.present(Some(window));
    }
}
