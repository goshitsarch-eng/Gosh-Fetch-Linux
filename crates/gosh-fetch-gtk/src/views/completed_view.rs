//! Completed view - download history page

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::models::DownloadObject;
use crate::widgets::DownloadRow;
use crate::window::GoshFetchWindow;
use gosh_fetch_core::{Download, DownloadsDb};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct CompletedView {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub list_box: RefCell<Option<gtk::ListBox>>,
        pub rows: RefCell<HashMap<String, DownloadRow>>,
        pub header_stats: RefCell<Option<gtk::Label>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CompletedView {
        const NAME: &'static str = "CompletedView";
        type Type = super::CompletedView;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for CompletedView {}
    impl WidgetImpl for CompletedView {}
    impl BoxImpl for CompletedView {}
}

glib::wrapper! {
    pub struct CompletedView(ObjectSubclass<imp::CompletedView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CompletedView {
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

        let title = adw::WindowTitle::new("Completed", "");
        header.set_title_widget(Some(&title));

        // Clear history button
        let clear_btn = gtk::Button::from_icon_name("user-trash-symbolic");
        clear_btn.set_tooltip_text(Some("Clear History"));
        let view = self.clone();
        clear_btn.connect_clicked(move |_| {
            view.clear_history();
        });
        header.pack_end(&clear_btn);

        self.append(&header);

        // Stats bar
        let stats_bar = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        stats_bar.set_margin_start(16);
        stats_bar.set_margin_end(16);
        stats_bar.set_margin_top(8);
        stats_bar.set_margin_bottom(8);

        let stats_label = gtk::Label::new(Some("0 completed downloads"));
        stats_label.add_css_class("dim-label");
        *self.imp().header_stats.borrow_mut() = Some(stats_label.clone());
        stats_bar.append(&stats_label);

        self.append(&stats_bar);

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
        empty_state.set_icon_name(Some("emblem-ok-symbolic"));
        empty_state.set_title("No Completed Downloads");
        empty_state.set_description(Some("Completed downloads will appear here"));

        // Stack to switch between list and empty state
        let stack = gtk::Stack::new();
        stack.add_named(&scrolled, Some("list"));
        stack.add_named(&empty_state, Some("empty"));
        stack.set_visible_child_name("empty");

        self.append(&stack);
    }

    pub fn add_download(&self, download: &Download) {
        let imp = self.imp();

        // Check if already exists
        if imp.rows.borrow().contains_key(&download.gid) {
            return;
        }

        let obj = DownloadObject::new(download);
        let row = DownloadRow::new();
        row.bind(&obj);

        // Connect open folder action
        let save_path = download.save_path.clone();
        row.connect_open_folder(move |_| {
            let _ = open::that(&save_path);
        });

        // Connect remove action
        let view = self.clone();
        let gid = download.gid.clone();
        row.connect_remove(move |_| {
            view.remove_download(&gid);
        });

        // Add to list (at the beginning)
        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            list_box.prepend(&row);
        }

        imp.rows.borrow_mut().insert(download.gid.clone(), row);
        self.update_empty_state();
        self.update_stats();
    }

    pub fn remove_download(&self, gid: &str) {
        let imp = self.imp();

        // Remove from UI
        if let Some(row) = imp.rows.borrow_mut().remove(gid) {
            if let Some(parent) = row.parent() {
                if let Some(list_box) = imp.list_box.borrow().as_ref() {
                    list_box.remove(&parent);
                }
            }
        }

        // Delete from database
        if let Some(window) = imp.window.borrow().as_ref() {
            if let Some(db) = window.db() {
                if let Err(e) = DownloadsDb::delete(db, gid) {
                    log::error!("Failed to delete download from database: {}", e);
                }
            }
        }

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

        // Add downloads
        for download in downloads {
            self.add_download(download);
        }

        self.update_empty_state();
        self.update_stats();
    }

    fn clear_history(&self) {
        let imp = self.imp();

        // Clear list
        if let Some(list_box) = imp.list_box.borrow().as_ref() {
            while let Some(child) = list_box.first_child() {
                list_box.remove(&child);
            }
        }
        imp.rows.borrow_mut().clear();

        // Clear from database
        if let Some(window) = imp.window.borrow().as_ref() {
            if let Some(db) = window.db() {
                if let Err(e) = DownloadsDb::clear_history(db) {
                    log::error!("Failed to clear history from database: {}", e);
                }
            }
        }

        self.update_empty_state();
        self.update_stats();
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
                "{} completed download{}",
                count,
                if count == 1 { "" } else { "s" }
            ));
        }
    }
}
