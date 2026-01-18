//! Downloads view - main download management page

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::models::DownloadObject;
use crate::widgets::DownloadRow;
use crate::window::GoshFetchWindow;
use gosh_fetch_core::{Download, DownloadState};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct DownloadsView {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub list_box: RefCell<Option<gtk::ListBox>>,
        pub rows: RefCell<HashMap<String, DownloadRow>>,
        pub row_statuses: RefCell<HashMap<String, DownloadState>>,
        pub empty_state: RefCell<Option<adw::StatusPage>>,
        pub header_stats: RefCell<Option<gtk::Label>>,
        pub filter: RefCell<Option<String>>,
        pub filter_buttons: RefCell<Vec<gtk::ToggleButton>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DownloadsView {
        const NAME: &'static str = "DownloadsView";
        type Type = super::DownloadsView;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for DownloadsView {}
    impl WidgetImpl for DownloadsView {}
    impl BoxImpl for DownloadsView {}
}

glib::wrapper! {
    pub struct DownloadsView(ObjectSubclass<imp::DownloadsView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl DownloadsView {
    pub fn new(window: &GoshFetchWindow) -> Self {
        let view: Self = glib::Object::new();
        *view.imp().window.borrow_mut() = Some(window.clone());
        view.setup_ui();
        view
    }

    fn setup_ui(&self) {
        self.set_orientation(gtk::Orientation::Vertical);
        self.set_spacing(0);

        // Header bar
        let header = adw::HeaderBar::new();

        let title = adw::WindowTitle::new("Downloads", "");
        header.set_title_widget(Some(&title));

        // Add download button
        let add_btn = gtk::Button::from_icon_name("list-add-symbolic");
        add_btn.set_tooltip_text(Some("Add Download"));
        add_btn.set_action_name(Some("win.add-download"));
        header.pack_start(&add_btn);

        // Pause/Resume all buttons
        let pause_all_btn = gtk::Button::from_icon_name("media-playback-pause-symbolic");
        pause_all_btn.set_tooltip_text(Some("Pause All"));
        pause_all_btn.set_action_name(Some("win.pause-all"));
        header.pack_end(&pause_all_btn);

        let resume_all_btn = gtk::Button::from_icon_name("media-playback-start-symbolic");
        resume_all_btn.set_tooltip_text(Some("Resume All"));
        resume_all_btn.set_action_name(Some("win.resume-all"));
        header.pack_end(&resume_all_btn);

        self.append(&header);

        // Stats bar
        let stats_bar = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        stats_bar.set_margin_start(16);
        stats_bar.set_margin_end(16);
        stats_bar.set_margin_top(8);
        stats_bar.set_margin_bottom(8);

        let stats_label = gtk::Label::new(Some("0 downloads"));
        stats_label.add_css_class("dim-label");
        *self.imp().header_stats.borrow_mut() = Some(stats_label.clone());
        stats_bar.append(&stats_label);

        self.append(&stats_bar);

        // Filter tabs
        let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        filter_box.set_margin_start(16);
        filter_box.set_margin_end(16);
        filter_box.set_margin_bottom(8);
        filter_box.add_css_class("linked");

        let filters = [
            ("All", None),
            ("Active", Some("active")),
            ("Paused", Some("paused")),
            ("Errors", Some("error")),
        ];

        let mut buttons = Vec::new();
        for (label, filter) in filters {
            let btn = gtk::ToggleButton::with_label(label);
            if filter.is_none() {
                btn.set_active(true);
            }
            buttons.push(btn.clone());
            filter_box.append(&btn);
        }

        // Store buttons and connect signals after all are created
        *self.imp().filter_buttons.borrow_mut() = buttons.clone();

        for (i, btn) in buttons.iter().enumerate() {
            let view = self.clone();
            let filter = filters[i].1.map(String::from);
            let all_buttons = buttons.clone();
            btn.connect_toggled(move |btn| {
                if btn.is_active() {
                    // Deactivate other buttons (mutually exclusive)
                    for other_btn in &all_buttons {
                        if other_btn != btn && other_btn.is_active() {
                            other_btn.set_active(false);
                        }
                    }
                    *view.imp().filter.borrow_mut() = filter.clone();
                    view.apply_filter();
                } else {
                    // Prevent deselecting the only active button
                    let any_active = all_buttons.iter().any(|b| b.is_active());
                    if !any_active {
                        btn.set_active(true);
                    }
                }
            });
        }

        self.append(&filter_box);

        // Scrolled window for list
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        // List box
        let list_box = gtk::ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::None);
        list_box.add_css_class("boxed-list");
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        list_box.set_margin_bottom(16);
        *self.imp().list_box.borrow_mut() = Some(list_box.clone());

        scrolled.set_child(Some(&list_box));

        // Empty state
        let empty_state = adw::StatusPage::new();
        empty_state.set_icon_name(Some("folder-download-symbolic"));
        empty_state.set_title("No Downloads");
        empty_state.set_description(Some("Press + to add a download"));
        empty_state.set_visible(true);
        *self.imp().empty_state.borrow_mut() = Some(empty_state.clone());

        // Stack to switch between list and empty state
        let stack = gtk::Stack::new();
        stack.add_named(&scrolled, Some("list"));
        stack.add_named(&empty_state, Some("empty"));
        stack.set_visible_child_name("empty");

        self.append(&stack);
    }

    pub fn add_download(&self, download: &Download) {
        let imp = self.imp();

        // Don't add completed downloads to this view
        if download.status == DownloadState::Complete {
            return;
        }

        // Check if already exists
        if imp.rows.borrow().contains_key(&download.gid) {
            self.update_download(&download.gid, download);
            return;
        }

        let obj = DownloadObject::new(download);
        let row = DownloadRow::new();
        row.bind(&obj);

        // Connect actions
        let window = imp.window.borrow().clone();
        let gid = download.gid.clone();
        row.connect_pause(move |_| {
            if let Some(window) = &window {
                window.pause_download(&gid);
            }
        });

        let window = imp.window.borrow().clone();
        let gid = download.gid.clone();
        row.connect_resume(move |_| {
            if let Some(window) = &window {
                window.resume_download(&gid);
            }
        });

        let window = imp.window.borrow().clone();
        let gid = download.gid.clone();
        row.connect_remove(move |_| {
            if let Some(window) = &window {
                window.remove_download(&gid, false);
            }
        });

        let save_path = download.save_path.clone();
        row.connect_open_folder(move |_| {
            let _ = open::that(&save_path);
        });

        // Add to list
        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            list_box.append(&row);
        }

        // Store status for filtering
        imp.row_statuses
            .borrow_mut()
            .insert(download.gid.clone(), download.status);
        imp.rows.borrow_mut().insert(download.gid.clone(), row);
        self.update_empty_state();
        self.update_stats();
        self.apply_filter();
    }

    pub fn update_download(&self, gid: &str, download: &Download) {
        let imp = self.imp();

        if let Some(row) = imp.rows.borrow().get(gid) {
            let obj = DownloadObject::new(download);
            row.bind(&obj);
        }

        // Update status for filtering
        imp.row_statuses
            .borrow_mut()
            .insert(gid.to_string(), download.status);

        self.update_stats();
        self.apply_filter();
    }

    pub fn remove_download(&self, gid: &str) {
        let imp = self.imp();

        if let Some(row) = imp.rows.borrow_mut().remove(gid) {
            // The row is wrapped in a ListBoxRow, so we need to remove the parent
            if let Some(parent) = row.parent() {
                if let Some(list_box) = imp.list_box.borrow().as_ref() {
                    list_box.remove(&parent);
                }
            }
        }

        imp.row_statuses.borrow_mut().remove(gid);

        self.update_empty_state();
        self.update_stats();
    }

    pub fn set_downloads(&self, downloads: &[Download]) {
        let imp = self.imp();

        // Clear existing
        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            while let Some(child) = list_box.first_child() {
                list_box.remove(&child);
            }
        }
        imp.rows.borrow_mut().clear();
        imp.row_statuses.borrow_mut().clear();

        // Add new downloads (excluding completed)
        for download in downloads {
            if download.status != DownloadState::Complete {
                self.add_download(download);
            }
        }

        self.update_empty_state();
        self.update_stats();
        self.apply_filter();
    }

    fn update_empty_state(&self) {
        let imp = self.imp();
        let is_empty = imp.rows.borrow().is_empty();

        if let Some(parent) = self.last_child() {
            if let Some(stack) = parent.downcast_ref::<gtk::Stack>() {
                stack.set_visible_child_name(if is_empty { "empty" } else { "list" });
            }
        }
    }

    fn update_stats(&self) {
        let imp = self.imp();
        let count = imp.rows.borrow().len();

        if let Some(label) = imp.header_stats.borrow().as_ref() {
            label.set_text(&format!(
                "{} download{}",
                count,
                if count == 1 { "" } else { "s" }
            ));
        }
    }

    fn apply_filter(&self) {
        let imp = self.imp();
        let filter = imp.filter.borrow().clone();
        let statuses = imp.row_statuses.borrow().clone();

        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            list_box.set_filter_func(move |row| {
                // No filter means show all
                let filter_str = match &filter {
                    None => return true,
                    Some(f) => f.as_str(),
                };

                // Get the download row and its gid
                let download_row = match row
                    .first_child()
                    .and_then(|w| w.downcast::<DownloadRow>().ok())
                {
                    Some(r) => r,
                    None => return true,
                };

                let gid = match download_row.gid() {
                    Some(g) => g,
                    None => return true,
                };

                // Get the status for this download
                let status = match statuses.get(&gid) {
                    Some(s) => *s,
                    None => return true,
                };

                // Filter based on status
                match filter_str {
                    "active" => matches!(status, DownloadState::Active | DownloadState::Waiting),
                    "paused" => status == DownloadState::Paused,
                    "error" => status == DownloadState::Error,
                    _ => true,
                }
            });
        }
    }
}
