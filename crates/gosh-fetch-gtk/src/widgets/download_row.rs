//! DownloadRow widget - displays a single download item

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

use crate::models::DownloadObject;
use gosh_fetch_core::{format_bytes, format_eta, format_speed, DownloadState, DownloadType};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct DownloadRow {
        pub download: RefCell<Option<DownloadObject>>,

        // UI elements (created manually)
        pub icon: RefCell<Option<gtk::Image>>,
        pub name_label: RefCell<Option<gtk::Label>>,
        pub status_label: RefCell<Option<gtk::Label>>,
        pub progress_bar: RefCell<Option<gtk::ProgressBar>>,
        pub progress_label: RefCell<Option<gtk::Label>>,
        pub speed_label: RefCell<Option<gtk::Label>>,
        pub eta_label: RefCell<Option<gtk::Label>>,
        pub peers_label: RefCell<Option<gtk::Label>>,
        pub pause_button: RefCell<Option<gtk::Button>>,
        pub resume_button: RefCell<Option<gtk::Button>>,
        pub remove_button: RefCell<Option<gtk::Button>>,
        pub open_button: RefCell<Option<gtk::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DownloadRow {
        const NAME: &'static str = "DownloadRow";
        type Type = super::DownloadRow;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for DownloadRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for DownloadRow {}
    impl BoxImpl for DownloadRow {}
}

glib::wrapper! {
    pub struct DownloadRow(ObjectSubclass<imp::DownloadRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl DownloadRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup_ui(&self) {
        self.set_orientation(gtk::Orientation::Vertical);
        self.set_spacing(8);
        self.set_margin_start(12);
        self.set_margin_end(12);
        self.set_margin_top(8);
        self.set_margin_bottom(8);
        self.add_css_class("card");

        // Main content box
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);

        // Icon
        let icon = gtk::Image::from_icon_name("folder-download-symbolic");
        icon.set_pixel_size(32);
        *self.imp().icon.borrow_mut() = Some(icon.clone());
        content.append(&icon);

        // Info section
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info_box.set_hexpand(true);

        // Name and status row
        let name_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let name_label = gtk::Label::new(Some("Download"));
        name_label.set_halign(gtk::Align::Start);
        name_label.set_hexpand(true);
        name_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        name_label.add_css_class("heading");
        *self.imp().name_label.borrow_mut() = Some(name_label.clone());
        name_row.append(&name_label);

        let status_label = gtk::Label::new(Some("Queued"));
        status_label.add_css_class("dim-label");
        *self.imp().status_label.borrow_mut() = Some(status_label.clone());
        name_row.append(&status_label);

        info_box.append(&name_row);

        // Progress bar
        let progress_bar = gtk::ProgressBar::new();
        progress_bar.set_fraction(0.0);
        progress_bar.set_margin_top(4);
        *self.imp().progress_bar.borrow_mut() = Some(progress_bar.clone());
        info_box.append(&progress_bar);

        // Stats row
        let stats_row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        stats_row.set_margin_top(4);

        let progress_label = gtk::Label::new(Some("0 B / 0 B"));
        progress_label.add_css_class("dim-label");
        progress_label.add_css_class("caption");
        *self.imp().progress_label.borrow_mut() = Some(progress_label.clone());
        stats_row.append(&progress_label);

        let speed_label = gtk::Label::new(Some("0 B/s"));
        speed_label.add_css_class("success");
        speed_label.add_css_class("caption");
        *self.imp().speed_label.borrow_mut() = Some(speed_label.clone());
        stats_row.append(&speed_label);

        let eta_label = gtk::Label::new(Some("--"));
        eta_label.add_css_class("dim-label");
        eta_label.add_css_class("caption");
        *self.imp().eta_label.borrow_mut() = Some(eta_label.clone());
        stats_row.append(&eta_label);

        let peers_label = gtk::Label::new(None);
        peers_label.add_css_class("dim-label");
        peers_label.add_css_class("caption");
        peers_label.set_visible(false);
        *self.imp().peers_label.borrow_mut() = Some(peers_label.clone());
        stats_row.append(&peers_label);

        info_box.append(&stats_row);
        content.append(&info_box);

        // Action buttons
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        let pause_button = gtk::Button::from_icon_name("media-playback-pause-symbolic");
        pause_button.set_tooltip_text(Some("Pause"));
        pause_button.add_css_class("flat");
        *self.imp().pause_button.borrow_mut() = Some(pause_button.clone());
        actions.append(&pause_button);

        let resume_button = gtk::Button::from_icon_name("media-playback-start-symbolic");
        resume_button.set_tooltip_text(Some("Resume"));
        resume_button.add_css_class("flat");
        resume_button.set_visible(false);
        *self.imp().resume_button.borrow_mut() = Some(resume_button.clone());
        actions.append(&resume_button);

        let open_button = gtk::Button::from_icon_name("folder-open-symbolic");
        open_button.set_tooltip_text(Some("Open Folder"));
        open_button.add_css_class("flat");
        open_button.set_visible(false);
        *self.imp().open_button.borrow_mut() = Some(open_button.clone());
        actions.append(&open_button);

        let remove_button = gtk::Button::from_icon_name("user-trash-symbolic");
        remove_button.set_tooltip_text(Some("Remove"));
        remove_button.add_css_class("flat");
        remove_button.add_css_class("destructive-action");
        *self.imp().remove_button.borrow_mut() = Some(remove_button.clone());
        actions.append(&remove_button);

        content.append(&actions);
        self.append(&content);
    }

    pub fn bind(&self, download: &DownloadObject) {
        *self.imp().download.borrow_mut() = Some(download.clone());
        self.update();
    }

    pub fn update(&self) {
        let imp = self.imp();
        let download = imp.download.borrow();

        if let Some(download) = download.as_ref() {
            // Update icon based on type
            if let Some(icon) = imp.icon.borrow().as_ref() {
                let icon_name = match download.download_type() {
                    DownloadType::Http | DownloadType::Ftp => "web-browser-symbolic",
                    DownloadType::Torrent | DownloadType::Magnet => {
                        "network-transmit-receive-symbolic"
                    }
                };
                icon.set_icon_name(Some(icon_name));
            }

            // Update name
            if let Some(label) = imp.name_label.borrow().as_ref() {
                label.set_text(&download.name());
            }

            // Update status
            if let Some(label) = imp.status_label.borrow().as_ref() {
                let status_text = match download.status() {
                    DownloadState::Active => {
                        if download.download_speed() > 0 {
                            "Downloading"
                        } else {
                            "Connecting"
                        }
                    }
                    DownloadState::Waiting => "Queued",
                    DownloadState::Paused => "Paused",
                    DownloadState::Complete => "Complete",
                    DownloadState::Error => "Error",
                    DownloadState::Removed => "Removed",
                };
                label.set_text(status_text);

                // Update status color
                label.remove_css_class("success");
                label.remove_css_class("warning");
                label.remove_css_class("error");
                match download.status() {
                    DownloadState::Active => label.add_css_class("success"),
                    DownloadState::Paused | DownloadState::Waiting => {
                        label.add_css_class("warning")
                    }
                    DownloadState::Error => label.add_css_class("error"),
                    _ => {}
                }
            }

            // Update progress bar
            if let Some(progress_bar) = imp.progress_bar.borrow().as_ref() {
                progress_bar.set_fraction(download.progress());
            }

            // Update progress label
            if let Some(label) = imp.progress_label.borrow().as_ref() {
                let completed = format_bytes(download.completed_size());
                let total = format_bytes(download.total_size());
                let percent = (download.progress() * 100.0) as u32;
                label.set_text(&format!("{} / {} ({}%)", completed, total, percent));
            }

            // Update speed
            if let Some(label) = imp.speed_label.borrow().as_ref() {
                if download.status() == DownloadState::Active {
                    let dl_speed = format_speed(download.download_speed());
                    let ul_speed = download.upload_speed();
                    if ul_speed > 0 {
                        label.set_text(&format!("↓ {} ↑ {}", dl_speed, format_speed(ul_speed)));
                    } else {
                        label.set_text(&format!("↓ {}", dl_speed));
                    }
                    label.set_visible(true);
                } else {
                    label.set_visible(false);
                }
            }

            // Update ETA
            if let Some(label) = imp.eta_label.borrow().as_ref() {
                if download.status() == DownloadState::Active && download.download_speed() > 0 {
                    let remaining = download
                        .total_size()
                        .saturating_sub(download.completed_size());
                    let eta = format_eta(remaining, download.download_speed());
                    label.set_text(&format!("ETA: {}", eta));
                    label.set_visible(true);
                } else {
                    label.set_visible(false);
                }
            }

            // Update peers/seeders (for torrents)
            if let Some(label) = imp.peers_label.borrow().as_ref() {
                let is_torrent = matches!(
                    download.download_type(),
                    DownloadType::Torrent | DownloadType::Magnet
                );
                if is_torrent && download.status() == DownloadState::Active {
                    let seeders = download.seeders();
                    let peers = download.connections();
                    label.set_text(&format!("Seeds: {} | Peers: {}", seeders, peers));
                    label.set_visible(true);
                } else {
                    label.set_visible(false);
                }
            }

            // Update button visibility
            let is_active = matches!(
                download.status(),
                DownloadState::Active | DownloadState::Waiting
            );
            let is_paused = download.status() == DownloadState::Paused;
            let is_complete = download.status() == DownloadState::Complete;

            if let Some(btn) = imp.pause_button.borrow().as_ref() {
                btn.set_visible(is_active);
            }
            if let Some(btn) = imp.resume_button.borrow().as_ref() {
                btn.set_visible(is_paused);
            }
            if let Some(btn) = imp.open_button.borrow().as_ref() {
                btn.set_visible(is_complete);
            }
        }
    }

    pub fn gid(&self) -> Option<String> {
        self.imp().download.borrow().as_ref().map(|d| d.gid())
    }

    pub fn connect_pause<F: Fn(&Self) + 'static>(&self, f: F) {
        if let Some(btn) = self.imp().pause_button.borrow().as_ref() {
            let row = self.clone();
            btn.connect_clicked(move |_| f(&row));
        }
    }

    pub fn connect_resume<F: Fn(&Self) + 'static>(&self, f: F) {
        if let Some(btn) = self.imp().resume_button.borrow().as_ref() {
            let row = self.clone();
            btn.connect_clicked(move |_| f(&row));
        }
    }

    pub fn connect_remove<F: Fn(&Self) + 'static>(&self, f: F) {
        if let Some(btn) = self.imp().remove_button.borrow().as_ref() {
            let row = self.clone();
            btn.connect_clicked(move |_| f(&row));
        }
    }

    pub fn connect_open_folder<F: Fn(&Self) + 'static>(&self, f: F) {
        if let Some(btn) = self.imp().open_button.borrow().as_ref() {
            let row = self.clone();
            btn.connect_clicked(move |_| f(&row));
        }
    }
}

impl Default for DownloadRow {
    fn default() -> Self {
        Self::new()
    }
}
