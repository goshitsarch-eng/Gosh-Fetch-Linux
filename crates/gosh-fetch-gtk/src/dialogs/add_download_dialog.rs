//! Add Download Dialog - dialog for adding new downloads with advanced options

use adw::prelude::*;
use adw::subclass::prelude::*;
use chrono::{Local, NaiveDateTime, TimeZone};
use gtk::{gio, glib};
use std::cell::RefCell;

use crate::window::GoshFetchWindow;
use gosh_fetch_core::DownloadOptions;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AddDownloadDialog {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub url_entry: RefCell<Option<gtk::Entry>>,
        pub magnet_text: RefCell<Option<gtk::TextView>>,
        pub torrent_path: RefCell<Option<String>>,
        pub torrent_label: RefCell<Option<gtk::Label>>,
        pub stack: RefCell<Option<adw::ViewStack>>,
        // Advanced options
        pub filename_entry: RefCell<Option<adw::EntryRow>>,
        pub location_row: RefCell<Option<adw::ActionRow>>,
        pub custom_location: RefCell<Option<String>>,
        pub speed_limit_row: RefCell<Option<adw::SpinRow>>,
        pub priority_row: RefCell<Option<adw::ComboRow>>,
        pub referer_entry: RefCell<Option<adw::EntryRow>>,
        pub cookies_entry: RefCell<Option<adw::EntryRow>>,
        pub checksum_type_row: RefCell<Option<adw::ComboRow>>,
        pub checksum_value_entry: RefCell<Option<adw::EntryRow>>,
        pub sequential_switch: RefCell<Option<adw::SwitchRow>>,
        pub advanced_expanded: RefCell<bool>,
        // Scheduling options
        pub schedule_switch: RefCell<Option<adw::SwitchRow>>,
        pub schedule_row: RefCell<Option<adw::ActionRow>>,
        pub scheduled_time: RefCell<Option<i64>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddDownloadDialog {
        const NAME: &'static str = "AddDownloadDialog";
        type Type = super::AddDownloadDialog;
        type ParentType = adw::Dialog;
    }

    impl ObjectImpl for AddDownloadDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for AddDownloadDialog {}
    impl AdwDialogImpl for AddDownloadDialog {}
}

glib::wrapper! {
    pub struct AddDownloadDialog(ObjectSubclass<imp::AddDownloadDialog>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AddDownloadDialog {
    pub fn new(window: &GoshFetchWindow) -> Self {
        let dialog: Self = glib::Object::new();
        *dialog.imp().window.borrow_mut() = Some(window.clone());
        dialog
    }

    fn setup_ui(&self) {
        self.set_title("Add Download");
        self.set_content_width(550);
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

        let add_btn = gtk::Button::with_label("Add");
        add_btn.add_css_class("suggested-action");
        add_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.add_download();
            }
        ));
        header.pack_end(&add_btn);

        content.append(&header);

        // Scrolled window for the content
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let inner_content = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // View stack switcher
        let stack = adw::ViewStack::new();
        *self.imp().stack.borrow_mut() = Some(stack.clone());

        let switcher = adw::ViewSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
        switcher.set_margin_start(16);
        switcher.set_margin_end(16);
        switcher.set_margin_top(8);
        switcher.set_margin_bottom(8);
        inner_content.append(&switcher);

        // URL tab
        let url_page = self.create_url_page();
        stack.add_titled_with_icon(&url_page, Some("url"), "URL", "web-browser-symbolic");

        // Magnet tab
        let magnet_page = self.create_magnet_page();
        stack.add_titled_with_icon(
            &magnet_page,
            Some("magnet"),
            "Magnet",
            "network-transmit-receive-symbolic",
        );

        // Torrent file tab
        let torrent_page = self.create_torrent_page();
        stack.add_titled_with_icon(
            &torrent_page,
            Some("torrent"),
            "Torrent File",
            "document-open-symbolic",
        );

        inner_content.append(&stack);

        // Advanced options (collapsible)
        let advanced_section = self.create_advanced_options();
        inner_content.append(&advanced_section);

        scrolled.set_child(Some(&inner_content));
        content.append(&scrolled);

        self.set_child(Some(&content));
    }

    fn create_url_page(&self) -> gtk::Box {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_start(16);
        page.set_margin_end(16);
        page.set_margin_top(16);
        page.set_margin_bottom(16);

        let label = gtk::Label::new(Some("Enter URL to download"));
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        page.append(&label);

        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some("https://example.com/file.zip"));
        entry.set_hexpand(true);
        *self.imp().url_entry.borrow_mut() = Some(entry.clone());
        page.append(&entry);

        let help = gtk::Label::new(Some("Supports HTTP, HTTPS, and magnet links"));
        help.set_halign(gtk::Align::Start);
        help.add_css_class("dim-label");
        help.add_css_class("caption");
        page.append(&help);

        page
    }

    fn create_magnet_page(&self) -> gtk::Box {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_start(16);
        page.set_margin_end(16);
        page.set_margin_top(16);
        page.set_margin_bottom(16);

        let label = gtk::Label::new(Some("Enter magnet link"));
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        page.append(&label);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_min_content_height(100);

        let text_view = gtk::TextView::new();
        text_view.set_wrap_mode(gtk::WrapMode::WordChar);
        text_view.set_accepts_tab(false);
        *self.imp().magnet_text.borrow_mut() = Some(text_view.clone());

        scrolled.set_child(Some(&text_view));
        page.append(&scrolled);

        let help = gtk::Label::new(Some("Paste your magnet:?xt=urn:btih:... link here"));
        help.set_halign(gtk::Align::Start);
        help.add_css_class("dim-label");
        help.add_css_class("caption");
        page.append(&help);

        page
    }

    fn create_torrent_page(&self) -> gtk::Box {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_start(16);
        page.set_margin_end(16);
        page.set_margin_top(16);
        page.set_margin_bottom(16);

        let label = gtk::Label::new(Some("Select a .torrent file"));
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        page.append(&label);

        let file_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let file_label = gtk::Label::new(Some("No file selected"));
        file_label.set_hexpand(true);
        file_label.set_halign(gtk::Align::Start);
        file_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        *self.imp().torrent_label.borrow_mut() = Some(file_label.clone());
        file_box.append(&file_label);

        let browse_btn = gtk::Button::with_label("Browse");
        browse_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.browse_torrent_file();
            }
        ));
        file_box.append(&browse_btn);

        page.append(&file_box);

        page
    }

    fn create_advanced_options(&self) -> adw::PreferencesGroup {
        let group = adw::PreferencesGroup::new();
        group.set_title("Advanced Options");
        group.set_margin_start(16);
        group.set_margin_end(16);
        group.set_margin_top(16);
        group.set_margin_bottom(16);

        // Save As (custom filename)
        let filename_entry = adw::EntryRow::new();
        filename_entry.set_title("Save As");
        filename_entry.set_text("");
        *self.imp().filename_entry.borrow_mut() = Some(filename_entry.clone());
        group.add(&filename_entry);

        // Download Location
        let location_row = adw::ActionRow::new();
        location_row.set_title("Download Location");
        location_row.set_subtitle("Default location");
        let browse_btn = gtk::Button::with_label("Browse");
        browse_btn.set_valign(gtk::Align::Center);
        let dialog_weak = self.downgrade();
        browse_btn.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.browse_download_location();
            }
        });
        location_row.add_suffix(&browse_btn);
        *self.imp().location_row.borrow_mut() = Some(location_row.clone());
        group.add(&location_row);

        // Speed Limit
        let speed_limit_row = adw::SpinRow::with_range(0.0, 100.0, 1.0);
        speed_limit_row.set_title("Speed Limit (MB/s)");
        speed_limit_row.set_subtitle("0 = Unlimited");
        speed_limit_row.set_value(0.0);
        *self.imp().speed_limit_row.borrow_mut() = Some(speed_limit_row.clone());
        group.add(&speed_limit_row);

        // Priority
        let priority_row = adw::ComboRow::new();
        priority_row.set_title("Priority");
        let priority_model = gtk::StringList::new(&["Normal", "Low", "High", "Critical"]);
        priority_row.set_model(Some(&priority_model));
        priority_row.set_selected(0);
        *self.imp().priority_row.borrow_mut() = Some(priority_row.clone());
        group.add(&priority_row);

        // Schedule download switch
        let schedule_switch = adw::SwitchRow::new();
        schedule_switch.set_title("Schedule Download");
        schedule_switch.set_subtitle("Start download at a specific time");
        schedule_switch.set_active(false);
        *self.imp().schedule_switch.borrow_mut() = Some(schedule_switch.clone());
        group.add(&schedule_switch);

        // Schedule time row (hidden by default)
        let schedule_row = adw::ActionRow::new();
        schedule_row.set_title("Scheduled Time");
        schedule_row.set_subtitle("Not set");
        schedule_row.set_visible(false);

        let time_btn = gtk::Button::with_label("Set Time");
        time_btn.set_valign(gtk::Align::Center);
        let dialog_weak = self.downgrade();
        time_btn.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.show_time_picker();
            }
        });
        schedule_row.add_suffix(&time_btn);
        *self.imp().schedule_row.borrow_mut() = Some(schedule_row.clone());
        group.add(&schedule_row);

        // Connect switch to show/hide time row
        let schedule_row_ref = schedule_row.clone();
        schedule_switch.connect_active_notify(move |switch| {
            schedule_row_ref.set_visible(switch.is_active());
        });

        // HTTP Options section
        let http_group = adw::PreferencesGroup::new();
        http_group.set_title("HTTP Options");
        http_group.set_margin_start(16);
        http_group.set_margin_end(16);
        http_group.set_margin_bottom(16);

        // Referer
        let referer_entry = adw::EntryRow::new();
        referer_entry.set_title("Referer URL");
        referer_entry.set_text("");
        *self.imp().referer_entry.borrow_mut() = Some(referer_entry.clone());
        http_group.add(&referer_entry);

        // Cookies
        let cookies_entry = adw::EntryRow::new();
        cookies_entry.set_title("Cookies");
        cookies_entry.set_text("");
        *self.imp().cookies_entry.borrow_mut() = Some(cookies_entry.clone());
        http_group.add(&cookies_entry);

        // Checksum verification
        let checksum_type_row = adw::ComboRow::new();
        checksum_type_row.set_title("Checksum Type");
        let checksum_model = gtk::StringList::new(&["None", "MD5", "SHA256"]);
        checksum_type_row.set_model(Some(&checksum_model));
        checksum_type_row.set_selected(0);
        *self.imp().checksum_type_row.borrow_mut() = Some(checksum_type_row.clone());
        http_group.add(&checksum_type_row);

        let checksum_value_entry = adw::EntryRow::new();
        checksum_value_entry.set_title("Checksum Value");
        checksum_value_entry.set_text("");
        *self.imp().checksum_value_entry.borrow_mut() = Some(checksum_value_entry.clone());
        http_group.add(&checksum_value_entry);

        // BitTorrent Options section
        let bt_group = adw::PreferencesGroup::new();
        bt_group.set_title("BitTorrent Options");
        bt_group.set_margin_start(16);
        bt_group.set_margin_end(16);
        bt_group.set_margin_bottom(16);

        // Sequential download
        let sequential_switch = adw::SwitchRow::new();
        sequential_switch.set_title("Sequential Download");
        sequential_switch.set_subtitle("Download pieces in order (useful for streaming)");
        sequential_switch.set_active(false);
        *self.imp().sequential_switch.borrow_mut() = Some(sequential_switch.clone());
        bt_group.add(&sequential_switch);

        // Create a container for all groups
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&group);
        container.append(&http_group);
        container.append(&bt_group);

        // Wrap in expander for collapsible behavior
        let expander_row = adw::ExpanderRow::new();
        expander_row.set_title("Advanced Options");
        expander_row.set_subtitle("Filename, location, speed limit, and more");
        expander_row.set_show_enable_switch(false);

        // Since ExpanderRow expects PreferencesRow children, we'll use a different approach
        // Return just the main group and add HTTP/BT groups directly
        // Actually, let's use a simpler approach - return the main group with all options

        group
    }

    fn browse_torrent_file(&self) {
        let dialog = gtk::FileDialog::new();
        dialog.set_title("Select Torrent File");

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.torrent");
        filter.set_name(Some("Torrent Files"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        let self_weak = self.downgrade();
        dialog.open(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            move |result| {
                if let Some(dialog) = self_weak.upgrade() {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            *dialog.imp().torrent_path.borrow_mut() = Some(path_str.clone());
                            if let Some(label) = dialog.imp().torrent_label.borrow().as_ref() {
                                label.set_text(&path_str);
                            }
                        }
                    }
                }
            },
        );
    }

    fn browse_download_location(&self) {
        let dialog = gtk::FileDialog::builder()
            .title("Select Download Location")
            .modal(true)
            .build();

        let self_weak = self.downgrade();
        dialog.select_folder(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            move |result| {
                if let Some(dialog) = self_weak.upgrade() {
                    if let Ok(folder) = result {
                        if let Some(path) = folder.path() {
                            let path_str = path.to_string_lossy().to_string();
                            *dialog.imp().custom_location.borrow_mut() = Some(path_str.clone());
                            if let Some(row) = dialog.imp().location_row.borrow().as_ref() {
                                row.set_subtitle(&path_str);
                            }
                        }
                    }
                }
            },
        );
    }

    fn show_time_picker(&self) {
        // Create a popover with date/time selection
        let popover = gtk::Popover::new();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);

        let title = gtk::Label::new(Some("Select Date and Time"));
        title.add_css_class("title-4");
        content.append(&title);

        // Calendar for date selection
        let calendar = gtk::Calendar::new();
        content.append(&calendar);

        // Time selection
        let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        time_box.set_halign(gtk::Align::Center);

        let now = Local::now();

        let hour_spin = gtk::SpinButton::with_range(0.0, 23.0, 1.0);
        hour_spin.set_value(now.format("%H").to_string().parse::<f64>().unwrap_or(12.0));
        hour_spin.set_wrap(true);
        hour_spin.set_width_chars(2);

        let minute_spin = gtk::SpinButton::with_range(0.0, 59.0, 1.0);
        minute_spin.set_value(now.format("%M").to_string().parse::<f64>().unwrap_or(0.0));
        minute_spin.set_wrap(true);
        minute_spin.set_width_chars(2);

        time_box.append(&hour_spin);
        time_box.append(&gtk::Label::new(Some(":")));
        time_box.append(&minute_spin);
        content.append(&time_box);

        // Confirm button
        let confirm_btn = gtk::Button::with_label("Set");
        confirm_btn.add_css_class("suggested-action");
        content.append(&confirm_btn);

        popover.set_child(Some(&content));

        // Connect confirm button
        let dialog_weak = self.downgrade();
        let popover_weak = popover.downgrade();
        confirm_btn.connect_clicked(move |_| {
            if let (Some(dialog), Some(popover)) = (dialog_weak.upgrade(), popover_weak.upgrade()) {
                let date = calendar.date();
                let hour = hour_spin.value() as u32;
                let minute = minute_spin.value() as u32;

                // Create datetime from selected values
                let datetime_str = format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:00",
                    date.year(),
                    date.month() as u32,
                    date.day_of_month(),
                    hour,
                    minute
                );

                if let Ok(naive) = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                {
                    if let Some(local) = Local.from_local_datetime(&naive).single() {
                        let timestamp = local.timestamp();
                        *dialog.imp().scheduled_time.borrow_mut() = Some(timestamp);

                        // Update the row subtitle
                        if let Some(row) = dialog.imp().schedule_row.borrow().as_ref() {
                            row.set_subtitle(&local.format("%Y-%m-%d %H:%M").to_string());
                        }
                    }
                }

                popover.popdown();
            }
        });

        // Show the popover relative to the schedule row
        if let Some(row) = self.imp().schedule_row.borrow().as_ref() {
            popover.set_parent(row);
            popover.popup();
        }
    }

    fn build_options(&self) -> Option<DownloadOptions> {
        let imp = self.imp();
        let mut opts = DownloadOptions::default();
        let mut has_options = false;

        // Custom filename
        if let Some(entry) = imp.filename_entry.borrow().as_ref() {
            let text = entry.text().to_string();
            if !text.is_empty() {
                opts.out = Some(text);
                has_options = true;
            }
        }

        // Custom location
        if let Some(path) = imp.custom_location.borrow().as_ref() {
            opts.dir = Some(path.clone());
            has_options = true;
        }

        // Speed limit
        if let Some(row) = imp.speed_limit_row.borrow().as_ref() {
            let val = row.value() as u64;
            if val > 0 {
                // Convert MB/s to bytes
                let bytes = val * 1024 * 1024;
                opts.max_download_limit = Some(format!("{}", bytes));
                has_options = true;
            }
        }

        // Priority
        if let Some(row) = imp.priority_row.borrow().as_ref() {
            let priority = match row.selected() {
                1 => Some("low".to_string()),
                2 => Some("high".to_string()),
                3 => Some("critical".to_string()),
                _ => None, // Normal is default
            };
            if priority.is_some() {
                opts.priority = priority;
                has_options = true;
            }
        }

        // Referer
        if let Some(entry) = imp.referer_entry.borrow().as_ref() {
            let text = entry.text().to_string();
            if !text.is_empty() {
                opts.referer = Some(text);
                has_options = true;
            }
        }

        // Cookies
        if let Some(entry) = imp.cookies_entry.borrow().as_ref() {
            let text = entry.text().to_string();
            if !text.is_empty() {
                opts.cookies = Some(text);
                has_options = true;
            }
        }

        // Checksum
        if let Some(type_row) = imp.checksum_type_row.borrow().as_ref() {
            let checksum_type = match type_row.selected() {
                1 => Some("md5".to_string()),
                2 => Some("sha256".to_string()),
                _ => None,
            };
            if let Some(ct) = checksum_type {
                if let Some(value_entry) = imp.checksum_value_entry.borrow().as_ref() {
                    let value = value_entry.text().to_string();
                    if !value.is_empty() {
                        opts.checksum_type = Some(ct);
                        opts.checksum_value = Some(value);
                        has_options = true;
                    }
                }
            }
        }

        // Sequential download
        if let Some(switch) = imp.sequential_switch.borrow().as_ref() {
            if switch.is_active() {
                opts.sequential = Some(true);
                has_options = true;
            }
        }

        // Scheduled start time
        if let Some(switch) = imp.schedule_switch.borrow().as_ref() {
            if switch.is_active() {
                if let Some(timestamp) = *imp.scheduled_time.borrow() {
                    opts.scheduled_start = Some(timestamp);
                    has_options = true;
                }
            }
        }

        if has_options {
            Some(opts)
        } else {
            None
        }
    }

    fn add_download(&self) {
        let imp = self.imp();

        // Get current tab
        let stack = imp.stack.borrow();
        let current_page = stack.as_ref().and_then(|s| s.visible_child_name());

        let window = imp.window.borrow();
        let window = match window.as_ref() {
            Some(w) => w,
            None => return,
        };

        let options = self.build_options();

        match current_page.as_ref().map(|s| s.as_str()) {
            Some("url") => {
                if let Some(entry) = imp.url_entry.borrow().as_ref() {
                    let url = entry.text().to_string();
                    if !url.is_empty() {
                        // Check if it's a magnet link
                        if url.starts_with("magnet:") {
                            window.add_magnet_with_options(&url, options);
                        } else {
                            window.add_url_with_options(&url, options);
                        }
                        self.close();
                    }
                }
            }

            Some("magnet") => {
                if let Some(text_view) = imp.magnet_text.borrow().as_ref() {
                    let buffer = text_view.buffer();
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    let uri = buffer.text(&start, &end, false).to_string();
                    if !uri.is_empty() && uri.starts_with("magnet:") {
                        window.add_magnet_with_options(&uri, options);
                        self.close();
                    }
                }
            }

            Some("torrent") => {
                if let Some(path) = imp.torrent_path.borrow().as_ref() {
                    // Read torrent file
                    if let Ok(data) = std::fs::read(path) {
                        window.add_torrent_with_options(&data, options);
                        self.close();
                    }
                }
            }

            _ => {}
        }
    }
}
