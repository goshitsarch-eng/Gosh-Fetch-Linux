//! Torrent Preview Dialog - shows torrent contents before adding

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

use crate::window::GoshFetchWindow;
use gosh_fetch_core::{DownloadOptions, TorrentFileEntry, TorrentInfo, format_bytes};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct TorrentPreviewDialog {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub torrent_info: RefCell<Option<TorrentInfo>>,
        pub torrent_data: RefCell<Option<Vec<u8>>>,
        pub file_checks: RefCell<Vec<gtk::CheckButton>>,
        pub selected_size_label: RefCell<Option<gtk::Label>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TorrentPreviewDialog {
        const NAME: &'static str = "TorrentPreviewDialog";
        type Type = super::TorrentPreviewDialog;
        type ParentType = adw::Dialog;
    }

    impl ObjectImpl for TorrentPreviewDialog {}
    impl WidgetImpl for TorrentPreviewDialog {}
    impl AdwDialogImpl for TorrentPreviewDialog {}
}

glib::wrapper! {
    pub struct TorrentPreviewDialog(ObjectSubclass<imp::TorrentPreviewDialog>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TorrentPreviewDialog {
    pub fn new(window: &GoshFetchWindow, torrent_data: Vec<u8>, info: TorrentInfo) -> Self {
        let dialog: Self = glib::Object::new();
        *dialog.imp().window.borrow_mut() = Some(window.clone());
        *dialog.imp().torrent_data.borrow_mut() = Some(torrent_data);
        *dialog.imp().torrent_info.borrow_mut() = Some(info);
        dialog.setup_ui();
        dialog
    }

    fn setup_ui(&self) {
        let info = self.imp().torrent_info.borrow();
        let info = match info.as_ref() {
            Some(i) => i,
            None => return,
        };

        self.set_title("Torrent Preview");
        self.set_content_width(600);
        self.set_content_height(500);

        // Main content box
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();
        header.set_show_start_title_buttons(false);
        header.set_show_end_title_buttons(false);

        let cancel_btn = gtk::Button::with_label("Cancel");
        cancel_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.close();
            }
        ));
        header.pack_start(&cancel_btn);

        let add_btn = gtk::Button::with_label("Add Download");
        add_btn.add_css_class("suggested-action");
        add_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.add_torrent();
            }
        ));
        header.pack_end(&add_btn);

        content.append(&header);

        // Scrolled window
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let inner_content = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Torrent info section
        let info_group = adw::PreferencesGroup::new();
        info_group.set_title("Torrent Information");
        info_group.set_margin_start(16);
        info_group.set_margin_end(16);
        info_group.set_margin_top(16);

        // Name
        let name_row = adw::ActionRow::new();
        name_row.set_title("Name");
        name_row.set_subtitle(&info.name);
        info_group.add(&name_row);

        // Size
        let size_row = adw::ActionRow::new();
        size_row.set_title("Total Size");
        size_row.set_subtitle(&format_bytes(info.total_size));
        info_group.add(&size_row);

        // Hash
        let hash_row = adw::ActionRow::new();
        hash_row.set_title("Info Hash");
        hash_row.set_subtitle(&info.info_hash);
        hash_row.set_subtitle_selectable(true);
        info_group.add(&hash_row);

        // Comment (if present)
        if let Some(ref comment) = info.comment {
            if !comment.is_empty() {
                let comment_row = adw::ActionRow::new();
                comment_row.set_title("Comment");
                comment_row.set_subtitle(comment);
                info_group.add(&comment_row);
            }
        }

        // Files count
        let files_count_row = adw::ActionRow::new();
        files_count_row.set_title("Files");
        files_count_row.set_subtitle(&format!("{} files", info.files.len()));
        info_group.add(&files_count_row);

        inner_content.append(&info_group);

        // File selection section
        let files_group = adw::PreferencesGroup::new();
        files_group.set_title("Select Files to Download");
        files_group.set_margin_start(16);
        files_group.set_margin_end(16);
        files_group.set_margin_top(16);

        // Select All / Select None buttons
        let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        btn_box.set_margin_bottom(8);

        let select_all_btn = gtk::Button::with_label("Select All");
        let dialog_weak = self.downgrade();
        select_all_btn.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.select_all(true);
            }
        });
        btn_box.append(&select_all_btn);

        let select_none_btn = gtk::Button::with_label("Select None");
        let dialog_weak = self.downgrade();
        select_none_btn.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.select_all(false);
            }
        });
        btn_box.append(&select_none_btn);

        // Selected size label
        let selected_label = gtk::Label::new(Some(&format!("Selected: {}", format_bytes(info.total_size))));
        selected_label.add_css_class("dim-label");
        selected_label.set_hexpand(true);
        selected_label.set_halign(gtk::Align::End);
        *self.imp().selected_size_label.borrow_mut() = Some(selected_label.clone());
        btn_box.append(&selected_label);

        files_group.set_header_suffix(Some(&btn_box));

        // File list
        let mut file_checks = Vec::new();
        for file in &info.files {
            let check = self.create_file_row(file);
            file_checks.push(check.clone());

            let row = adw::ActionRow::new();
            row.set_title(&file.path);
            row.set_subtitle(&format_bytes(file.length));
            row.add_prefix(&check);
            row.set_activatable_widget(Some(&check));
            files_group.add(&row);
        }
        *self.imp().file_checks.borrow_mut() = file_checks;

        inner_content.append(&files_group);

        scrolled.set_child(Some(&inner_content));
        content.append(&scrolled);

        self.set_child(Some(&content));
    }

    fn create_file_row(&self, _file: &TorrentFileEntry) -> gtk::CheckButton {
        let check = gtk::CheckButton::new();
        check.set_active(true);

        let dialog_weak = self.downgrade();
        check.connect_toggled(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.update_selected_size();
            }
        });

        check
    }

    fn select_all(&self, selected: bool) {
        let checks = self.imp().file_checks.borrow();
        for check in checks.iter() {
            check.set_active(selected);
        }
        self.update_selected_size();
    }

    fn update_selected_size(&self) {
        let info = self.imp().torrent_info.borrow();
        let checks = self.imp().file_checks.borrow();

        if let Some(info) = info.as_ref() {
            let mut total: u64 = 0;
            for (i, check) in checks.iter().enumerate() {
                if check.is_active() && i < info.files.len() {
                    total += info.files[i].length;
                }
            }

            if let Some(label) = self.imp().selected_size_label.borrow().as_ref() {
                label.set_text(&format!("Selected: {}", format_bytes(total)));
            }
        }
    }

    fn add_torrent(&self) {
        let imp = self.imp();

        // Get selected file indices
        let info = imp.torrent_info.borrow();
        let checks = imp.file_checks.borrow();

        let selected_indices: Vec<usize> = if let Some(info) = info.as_ref() {
            checks.iter()
                .enumerate()
                .filter(|(i, check)| check.is_active() && *i < info.files.len())
                .map(|(i, _)| info.files[i].index)
                .collect()
        } else {
            Vec::new()
        };

        // Build options with selected files
        let options = if selected_indices.is_empty() ||
            selected_indices.len() == info.as_ref().map(|i| i.files.len()).unwrap_or(0) {
            None // All files selected, no need to filter
        } else {
            let file_list = selected_indices.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            Some(DownloadOptions {
                select_file: Some(file_list),
                ..Default::default()
            })
        };

        // Add the torrent
        if let Some(window) = imp.window.borrow().as_ref() {
            if let Some(data) = imp.torrent_data.borrow().as_ref() {
                window.add_torrent_with_options(data, options);
            }
        }

        self.close();
    }
}
