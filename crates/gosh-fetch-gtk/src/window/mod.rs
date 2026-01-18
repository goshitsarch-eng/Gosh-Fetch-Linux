//! Main window module

mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use notify_rust::Notification;

use gosh_fetch_core::{Database, Download, EngineCommand, UiMessage};

glib::wrapper! {
    pub struct GoshFetchWindow(ObjectSubclass<imp::GoshFetchWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GoshFetchWindow {
    pub fn new(
        app: &crate::application::GoshFetchApplication,
        db: Database,
        cmd_sender: async_channel::Sender<EngineCommand>,
    ) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("default-width", 1200)
            .property("default-height", 800)
            .property("title", "Gosh-Fetch")
            .build();

        window.imp().db.set(db).expect("Failed to set database");
        window
            .imp()
            .cmd_sender
            .set(cmd_sender)
            .expect("Failed to set command sender");

        window.setup_ui();
        window.setup_actions();
        window.start_polling();

        window
    }

    fn setup_ui(&self) {
        self.imp().setup_ui(self);
    }

    fn setup_actions(&self) {
        // Add download action
        let add_action = gio::ActionEntry::builder("add-download")
            .activate(|window: &Self, _, _| {
                window.show_add_download_dialog();
            })
            .build();

        // Pause all action
        let pause_all_action = gio::ActionEntry::builder("pause-all")
            .activate(|window: &Self, _, _| {
                window.send_engine_command(EngineCommand::PauseAll);
            })
            .build();

        // Resume all action
        let resume_all_action = gio::ActionEntry::builder("resume-all")
            .activate(|window: &Self, _, _| {
                window.send_engine_command(EngineCommand::ResumeAll);
            })
            .build();

        self.add_action_entries([add_action, pause_all_action, resume_all_action]);
    }

    fn start_polling(&self) {
        // Request initial downloads list
        self.send_engine_command(EngineCommand::RefreshDownloads);
        self.send_engine_command(EngineCommand::RefreshStats);

        // Set up periodic polling (1 second)
        let window = self.downgrade();
        glib::timeout_add_seconds_local(1, move || {
            if let Some(window) = window.upgrade() {
                window.send_engine_command(EngineCommand::RefreshDownloads);
                window.send_engine_command(EngineCommand::RefreshStats);
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    pub fn handle_ui_message(&self, msg: UiMessage) {
        match msg {
            UiMessage::EngineReady => {
                log::info!("Download engine ready");
            }

            UiMessage::DownloadAdded(download) => {
                self.imp().add_download(&download);
            }

            UiMessage::DownloadUpdated(gid, download) => {
                self.imp().update_download(&gid, &download);
            }

            UiMessage::DownloadRemoved(gid) => {
                self.imp().remove_download(&gid);
            }

            UiMessage::DownloadCompleted(download) => {
                self.imp().update_download(&download.gid, &download);
                self.imp().add_to_completed(&download);
                self.send_download_notification(&download);
            }

            UiMessage::DownloadFailed(gid, error) => {
                log::error!("Download {} failed: {}", gid, error);
                self.show_error(&format!("Download failed: {}", error));
            }

            UiMessage::StatsUpdated(stats) => {
                self.imp().update_stats(&stats);
            }

            UiMessage::DownloadsList(downloads) => {
                self.imp().set_downloads(downloads);
            }

            UiMessage::Error(error) => {
                log::error!("Error: {}", error);
                self.show_error(&error);
            }
        }
    }

    fn show_error(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(5);
        self.imp().toast_overlay.get().unwrap().add_toast(toast);
    }

    fn show_add_download_dialog(&self) {
        self.imp().show_add_download_dialog(self);
    }

    fn send_engine_command(&self, cmd: EngineCommand) {
        if let Some(sender) = self.imp().cmd_sender.get() {
            let _ = sender.send_blocking(cmd);
        }
    }

    fn send_download_notification(&self, download: &Download) {
        // Check if notifications are enabled in settings
        if let Some(app) = self.application() {
            if let Some(gosh_app) = app.downcast_ref::<crate::application::GoshFetchApplication>() {
                let settings = gosh_app.settings();
                if !settings.enable_notifications {
                    return;
                }
            }
        }

        // Send desktop notification
        if let Err(e) = Notification::new()
            .summary("Download Complete")
            .body(&format!("{} has finished downloading", download.name))
            .icon("folder-download-symbolic")
            .appname("Gosh-Fetch")
            .show()
        {
            log::warn!("Failed to show notification: {}", e);
        }
    }

    pub fn db(&self) -> Option<&Database> {
        self.imp().db.get()
    }

    pub fn pause_download(&self, gid: &str) {
        self.send_engine_command(EngineCommand::Pause(gid.to_string()));
    }

    pub fn resume_download(&self, gid: &str) {
        self.send_engine_command(EngineCommand::Resume(gid.to_string()));
    }

    pub fn remove_download(&self, gid: &str, delete_files: bool) {
        self.send_engine_command(EngineCommand::Remove {
            gid: gid.to_string(),
            delete_files,
        });
    }

    pub fn add_url(&self, url: &str) {
        self.send_engine_command(EngineCommand::AddDownload {
            url: url.to_string(),
            options: None,
        });
    }

    pub fn add_url_with_options(&self, url: &str, options: Option<gosh_fetch_core::DownloadOptions>) {
        self.send_engine_command(EngineCommand::AddDownload {
            url: url.to_string(),
            options,
        });
    }

    pub fn add_magnet(&self, uri: &str) {
        self.send_engine_command(EngineCommand::AddMagnet {
            uri: uri.to_string(),
            options: None,
        });
    }

    pub fn add_magnet_with_options(&self, uri: &str, options: Option<gosh_fetch_core::DownloadOptions>) {
        self.send_engine_command(EngineCommand::AddMagnet {
            uri: uri.to_string(),
            options,
        });
    }

    pub fn add_torrent(&self, data: &[u8]) {
        self.send_engine_command(EngineCommand::AddTorrent {
            data: data.to_vec(),
            options: None,
        });
    }

    pub fn add_torrent_with_options(&self, data: &[u8], options: Option<gosh_fetch_core::DownloadOptions>) {
        self.send_engine_command(EngineCommand::AddTorrent {
            data: data.to_vec(),
            options,
        });
    }
}
