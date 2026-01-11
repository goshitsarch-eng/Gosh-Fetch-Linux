//! Settings view - application configuration page

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{OnceCell, RefCell};

use crate::window::GoshFetchWindow;
use gosh_fetch_core::{get_user_agent_presets, Settings, SettingsDb};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SettingsView {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub settings: RefCell<Settings>,
        pub download_row: OnceCell<adw::ActionRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsView {
        const NAME: &'static str = "SettingsView";
        type Type = super::SettingsView;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for SettingsView {}
    impl WidgetImpl for SettingsView {}
    impl BoxImpl for SettingsView {}
}

glib::wrapper! {
    pub struct SettingsView(ObjectSubclass<imp::SettingsView>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SettingsView {
    pub fn new(window: &GoshFetchWindow) -> Self {
        let view: Self = glib::Object::new();
        *view.imp().window.borrow_mut() = Some(window.clone());

        // Load settings from database
        if let Some(db) = window.db() {
            if let Ok(settings) = SettingsDb::load(db) {
                *view.imp().settings.borrow_mut() = settings;
            }
        }

        view.setup_ui();
        view
    }

    fn setup_ui(&self) {
        self.set_orientation(gtk::Orientation::Vertical);
        self.set_spacing(0);

        // Header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Settings", "");
        header.set_title_widget(Some(&title));
        self.append(&header);

        // Scrolled window
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        // Preferences page
        let prefs_page = adw::PreferencesPage::new();

        // General group
        let general_group = adw::PreferencesGroup::new();
        general_group.set_title("General");

        // Download location
        let download_row = adw::ActionRow::new();
        download_row.set_title("Download Location");
        let settings = self.imp().settings.borrow();
        download_row.set_subtitle(&settings.download_path);

        let browse_btn = gtk::Button::with_label("Browse");
        browse_btn.set_valign(gtk::Align::Center);
        let view = self.clone();
        browse_btn.connect_clicked(move |_| {
            view.browse_download_path();
        });
        download_row.add_suffix(&browse_btn);
        general_group.add(&download_row);
        let _ = self.imp().download_row.set(download_row);

        // Notifications
        let notif_row = adw::SwitchRow::new();
        notif_row.set_title("Enable Notifications");
        notif_row.set_subtitle("Show notifications when downloads complete");
        notif_row.set_active(settings.enable_notifications);
        general_group.add(&notif_row);

        // Close to tray
        let tray_row = adw::SwitchRow::new();
        tray_row.set_title("Close to Tray");
        tray_row.set_subtitle("Minimize to system tray instead of quitting");
        tray_row.set_active(settings.close_to_tray);
        general_group.add(&tray_row);

        // Delete files on remove
        let delete_row = adw::SwitchRow::new();
        delete_row.set_title("Delete Files on Remove");
        delete_row.set_subtitle("Delete downloaded files when removing from list");
        delete_row.set_active(settings.delete_files_on_remove);
        general_group.add(&delete_row);

        prefs_page.add(&general_group);

        // Connection group
        let conn_group = adw::PreferencesGroup::new();
        conn_group.set_title("Connection");

        // Concurrent downloads
        let concurrent_row = adw::SpinRow::with_range(1.0, 20.0, 1.0);
        concurrent_row.set_title("Concurrent Downloads");
        concurrent_row.set_subtitle("Maximum number of simultaneous downloads");
        concurrent_row.set_value(settings.max_concurrent_downloads as f64);
        conn_group.add(&concurrent_row);

        // Connections per server
        let conns_row = adw::SpinRow::with_range(1.0, 16.0, 1.0);
        conns_row.set_title("Connections per Server");
        conns_row.set_subtitle("Maximum connections to a single server");
        conns_row.set_value(settings.max_connections_per_server as f64);
        conn_group.add(&conns_row);

        // Split count
        let split_row = adw::SpinRow::with_range(1.0, 64.0, 1.0);
        split_row.set_title("Split Count");
        split_row.set_subtitle("Number of segments per download");
        split_row.set_value(settings.split_count as f64);
        conn_group.add(&split_row);

        // Download speed limit
        let dl_limit_row = adw::ActionRow::new();
        dl_limit_row.set_title("Download Speed Limit");
        dl_limit_row.set_subtitle("0 = Unlimited");

        let dl_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        dl_scale.set_width_request(200);
        dl_scale.set_value(settings.download_speed_limit as f64 / 1024.0 / 1024.0);
        dl_scale.set_valign(gtk::Align::Center);
        dl_limit_row.add_suffix(&dl_scale);

        let dl_label = gtk::Label::new(Some(&format!(
            "{} MB/s",
            settings.download_speed_limit / 1024 / 1024
        )));
        dl_label.set_width_chars(10);
        let dl_label_clone = dl_label.clone();
        dl_scale.connect_value_changed(move |scale| {
            let val = scale.value() as u64;
            if val == 0 {
                dl_label_clone.set_text("Unlimited");
            } else {
                dl_label_clone.set_text(&format!("{} MB/s", val));
            }
        });
        dl_limit_row.add_suffix(&dl_label);
        conn_group.add(&dl_limit_row);

        // Upload speed limit
        let ul_limit_row = adw::ActionRow::new();
        ul_limit_row.set_title("Upload Speed Limit");
        ul_limit_row.set_subtitle("0 = Unlimited");

        let ul_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        ul_scale.set_width_request(200);
        ul_scale.set_value(settings.upload_speed_limit as f64 / 1024.0 / 1024.0);
        ul_scale.set_valign(gtk::Align::Center);
        ul_limit_row.add_suffix(&ul_scale);

        let ul_label = gtk::Label::new(Some(&format!(
            "{} MB/s",
            settings.upload_speed_limit / 1024 / 1024
        )));
        ul_label.set_width_chars(10);
        let ul_label_clone = ul_label.clone();
        ul_scale.connect_value_changed(move |scale| {
            let val = scale.value() as u64;
            if val == 0 {
                ul_label_clone.set_text("Unlimited");
            } else {
                ul_label_clone.set_text(&format!("{} MB/s", val));
            }
        });
        ul_limit_row.add_suffix(&ul_label);
        conn_group.add(&ul_limit_row);

        prefs_page.add(&conn_group);

        // User Agent group
        let ua_group = adw::PreferencesGroup::new();
        ua_group.set_title("User Agent");

        let ua_row = adw::ComboRow::new();
        ua_row.set_title("User Agent");
        ua_row.set_subtitle("Identify as this browser when downloading");

        let presets = get_user_agent_presets();
        let model =
            gtk::StringList::new(&presets.iter().map(|(name, _)| *name).collect::<Vec<_>>());
        ua_row.set_model(Some(&model));

        // Find current selection
        let current_ua = &settings.user_agent;
        let selected = presets
            .iter()
            .position(|(_, ua)| ua == current_ua)
            .unwrap_or(0);
        ua_row.set_selected(selected as u32);

        ua_group.add(&ua_row);
        prefs_page.add(&ua_group);

        // BitTorrent group
        let bt_group = adw::PreferencesGroup::new();
        bt_group.set_title("BitTorrent");

        // Auto update trackers
        let tracker_row = adw::SwitchRow::new();
        tracker_row.set_title("Auto-Update Tracker List");
        tracker_row.set_subtitle("Automatically fetch updated tracker list daily");
        tracker_row.set_active(settings.auto_update_trackers);
        bt_group.add(&tracker_row);

        // Update trackers button
        let update_row = adw::ActionRow::new();
        update_row.set_title("Update Tracker List");
        update_row.set_subtitle("Fetch the latest tracker list now");

        let update_btn = gtk::Button::with_label("Update Now");
        update_btn.set_valign(gtk::Align::Center);
        update_btn.connect_clicked(|_| {
            // TODO: Trigger tracker update
        });
        update_row.add_suffix(&update_btn);
        bt_group.add(&update_row);

        prefs_page.add(&bt_group);
        drop(settings);

        scrolled.set_child(Some(&prefs_page));
        self.append(&scrolled);
    }

    fn browse_download_path(&self) {
        let window = self.imp().window.borrow();
        let Some(window) = window.as_ref() else {
            return;
        };

        let dialog = gtk::FileDialog::builder()
            .title("Select Download Location")
            .modal(true)
            .build();

        let view = self.clone();
        dialog.select_folder(
            Some(window),
            None::<&gtk::gio::Cancellable>,
            move |result| {
                if let Ok(folder) = result {
                    if let Some(path) = folder.path() {
                        let path_str = path.to_string_lossy().to_string();
                        view.update_download_path(&path_str);
                    }
                }
            },
        );
    }

    fn update_download_path(&self, path: &str) {
        // Update settings
        self.imp().settings.borrow_mut().download_path = path.to_string();

        // Update the UI
        if let Some(row) = self.imp().download_row.get() {
            row.set_subtitle(path);
        }
    }
}
