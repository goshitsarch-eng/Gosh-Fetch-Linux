//! Settings view - application configuration page

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::{OnceCell, RefCell};

use crate::window::GoshFetchWindow;
use gosh_fetch_core::{get_user_agent_presets, Settings, SettingsDb, TrackersDb, TrackerUpdater};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SettingsView {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub settings: RefCell<Settings>,
        pub download_row: OnceCell<adw::ActionRow>,
        pub toast_overlay: OnceCell<adw::ToastOverlay>,
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

        // Toast overlay for notifications
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_vexpand(true);
        let _ = self.imp().toast_overlay.set(toast_overlay.clone());

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
        let view = self.clone();
        notif_row.connect_active_notify(move |row| {
            view.save_setting("enable_notifications", if row.is_active() { "true" } else { "false" });
        });
        general_group.add(&notif_row);

        // Close to tray
        let tray_row = adw::SwitchRow::new();
        tray_row.set_title("Close to Tray");
        tray_row.set_subtitle("Minimize to system tray instead of quitting");
        tray_row.set_active(settings.close_to_tray);
        let view = self.clone();
        tray_row.connect_active_notify(move |row| {
            view.save_setting("close_to_tray", if row.is_active() { "true" } else { "false" });
        });
        general_group.add(&tray_row);

        // Delete files on remove
        let delete_row = adw::SwitchRow::new();
        delete_row.set_title("Delete Files on Remove");
        delete_row.set_subtitle("Delete downloaded files when removing from list");
        delete_row.set_active(settings.delete_files_on_remove);
        let view = self.clone();
        delete_row.connect_active_notify(move |row| {
            view.save_setting("delete_files_on_remove", if row.is_active() { "true" } else { "false" });
        });
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
        let view = self.clone();
        concurrent_row.connect_value_notify(move |row| {
            view.save_setting("max_concurrent_downloads", &(row.value() as u32).to_string());
        });
        conn_group.add(&concurrent_row);

        // Connections per server
        let conns_row = adw::SpinRow::with_range(1.0, 16.0, 1.0);
        conns_row.set_title("Connections per Server");
        conns_row.set_subtitle("Maximum connections to a single server");
        conns_row.set_value(settings.max_connections_per_server as f64);
        let view = self.clone();
        conns_row.connect_value_notify(move |row| {
            view.save_setting("max_connections_per_server", &(row.value() as u32).to_string());
        });
        conn_group.add(&conns_row);

        // Split count
        let split_row = adw::SpinRow::with_range(1.0, 64.0, 1.0);
        split_row.set_title("Split Count");
        split_row.set_subtitle("Number of segments per download");
        split_row.set_value(settings.split_count as f64);
        let view = self.clone();
        split_row.connect_value_notify(move |row| {
            view.save_setting("split_count", &(row.value() as u32).to_string());
        });
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

        let dl_label = gtk::Label::new(Some(if settings.download_speed_limit == 0 {
            "Unlimited".to_string()
        } else {
            format!("{} MB/s", settings.download_speed_limit / 1024 / 1024)
        }.as_str()));
        dl_label.set_width_chars(10);
        let dl_label_clone = dl_label.clone();
        let view = self.clone();
        dl_scale.connect_value_changed(move |scale| {
            let val = scale.value() as u64;
            if val == 0 {
                dl_label_clone.set_text("Unlimited");
            } else {
                dl_label_clone.set_text(&format!("{} MB/s", val));
            }
            // Save as bytes (MB * 1024 * 1024)
            let bytes = val * 1024 * 1024;
            view.save_setting("download_speed_limit", &bytes.to_string());
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

        let ul_label = gtk::Label::new(Some(if settings.upload_speed_limit == 0 {
            "Unlimited".to_string()
        } else {
            format!("{} MB/s", settings.upload_speed_limit / 1024 / 1024)
        }.as_str()));
        ul_label.set_width_chars(10);
        let ul_label_clone = ul_label.clone();
        let view = self.clone();
        ul_scale.connect_value_changed(move |scale| {
            let val = scale.value() as u64;
            if val == 0 {
                ul_label_clone.set_text("Unlimited");
            } else {
                ul_label_clone.set_text(&format!("{} MB/s", val));
            }
            // Save as bytes (MB * 1024 * 1024)
            let bytes = val * 1024 * 1024;
            view.save_setting("upload_speed_limit", &bytes.to_string());
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

        let view = self.clone();
        let presets_clone = presets.clone();
        ua_row.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            if idx < presets_clone.len() {
                view.save_setting("user_agent", presets_clone[idx].1);
            }
        });

        ua_group.add(&ua_row);
        prefs_page.add(&ua_group);

        // Proxy group
        let proxy_group = adw::PreferencesGroup::new();
        proxy_group.set_title("Proxy");

        // Enable proxy
        let proxy_enabled_row = adw::SwitchRow::new();
        proxy_enabled_row.set_title("Enable Proxy");
        proxy_enabled_row.set_subtitle("Route downloads through a proxy server");
        proxy_enabled_row.set_active(settings.proxy_enabled);
        let view = self.clone();
        proxy_enabled_row.connect_active_notify(move |row| {
            view.save_setting("proxy_enabled", if row.is_active() { "true" } else { "false" });
        });
        proxy_group.add(&proxy_enabled_row);

        // Proxy type
        let proxy_type_row = adw::ComboRow::new();
        proxy_type_row.set_title("Proxy Type");
        let proxy_types = gtk::StringList::new(&["HTTP", "HTTPS", "SOCKS5"]);
        proxy_type_row.set_model(Some(&proxy_types));
        let type_idx = match settings.proxy_type.as_str() {
            "https" => 1,
            "socks5" => 2,
            _ => 0,
        };
        proxy_type_row.set_selected(type_idx);
        let view = self.clone();
        proxy_type_row.connect_selected_notify(move |row| {
            let proxy_type = match row.selected() {
                1 => "https",
                2 => "socks5",
                _ => "http",
            };
            view.save_setting("proxy_type", proxy_type);
        });
        proxy_group.add(&proxy_type_row);

        // Proxy URL
        let proxy_url_row = adw::EntryRow::new();
        proxy_url_row.set_title("Proxy URL");
        proxy_url_row.set_text(&settings.proxy_url);
        let view = self.clone();
        proxy_url_row.connect_changed(move |row| {
            view.save_setting("proxy_url", &row.text());
        });
        proxy_group.add(&proxy_url_row);

        // Proxy username
        let proxy_user_row = adw::EntryRow::new();
        proxy_user_row.set_title("Username (optional)");
        if let Some(ref user) = settings.proxy_user {
            proxy_user_row.set_text(user);
        }
        let view = self.clone();
        proxy_user_row.connect_changed(move |row| {
            view.save_setting("proxy_user", &row.text());
        });
        proxy_group.add(&proxy_user_row);

        // Proxy password
        let proxy_pass_row = adw::PasswordEntryRow::new();
        proxy_pass_row.set_title("Password (optional)");
        if let Some(ref pass) = settings.proxy_pass {
            proxy_pass_row.set_text(pass);
        }
        let view = self.clone();
        proxy_pass_row.connect_changed(move |row| {
            view.save_setting("proxy_pass", &row.text());
        });
        proxy_group.add(&proxy_pass_row);

        prefs_page.add(&proxy_group);

        // Advanced Connection group
        let adv_conn_group = adw::PreferencesGroup::new();
        adv_conn_group.set_title("Advanced Connection");

        // Minimum segment size
        let min_seg_row = adw::SpinRow::with_range(256.0, 10240.0, 256.0);
        min_seg_row.set_title("Minimum Segment Size (KB)");
        min_seg_row.set_subtitle("Minimum size of each download segment");
        min_seg_row.set_value(settings.min_segment_size as f64);
        let view = self.clone();
        min_seg_row.connect_value_notify(move |row| {
            view.save_setting("min_segment_size", &(row.value() as u32).to_string());
        });
        adv_conn_group.add(&min_seg_row);

        prefs_page.add(&adv_conn_group);

        // BitTorrent group
        let bt_group = adw::PreferencesGroup::new();
        bt_group.set_title("BitTorrent");

        // File preallocation
        let prealloc_row = adw::ComboRow::new();
        prealloc_row.set_title("File Preallocation");
        prealloc_row.set_subtitle("How to allocate disk space before downloading");
        let prealloc_model = gtk::StringList::new(&["None", "Sparse", "Full"]);
        prealloc_row.set_model(Some(&prealloc_model));
        let prealloc_idx = match settings.bt_preallocation.as_str() {
            "none" => 0,
            "sparse" => 1,
            "full" => 2,
            _ => 1,
        };
        prealloc_row.set_selected(prealloc_idx);
        let view = self.clone();
        prealloc_row.connect_selected_notify(move |row| {
            let mode = match row.selected() {
                0 => "none",
                2 => "full",
                _ => "sparse",
            };
            view.save_setting("bt_preallocation", mode);
        });
        bt_group.add(&prealloc_row);

        // Auto update trackers
        let tracker_row = adw::SwitchRow::new();
        tracker_row.set_title("Auto-Update Tracker List");
        tracker_row.set_subtitle("Automatically fetch updated tracker list daily");
        tracker_row.set_active(settings.auto_update_trackers);
        let view = self.clone();
        tracker_row.connect_active_notify(move |row| {
            view.save_setting("auto_update_trackers", if row.is_active() { "true" } else { "false" });
        });
        bt_group.add(&tracker_row);

        // Update trackers button
        let update_row = adw::ActionRow::new();
        update_row.set_title("Update Tracker List");
        update_row.set_subtitle("Fetch the latest tracker list now");

        let update_btn = gtk::Button::with_label("Update Now");
        update_btn.set_valign(gtk::Align::Center);

        let view = self.clone();
        update_btn.connect_clicked(move |btn| {
            view.update_trackers(btn);
        });
        update_row.add_suffix(&update_btn);
        bt_group.add(&update_row);

        prefs_page.add(&bt_group);
        drop(settings);

        scrolled.set_child(Some(&prefs_page));
        toast_overlay.set_child(Some(&scrolled));
        self.append(&toast_overlay);
    }

    fn update_trackers(&self, btn: &gtk::Button) {
        let imp = self.imp();
        let db = match imp.window.borrow().as_ref().and_then(|w| w.db().cloned()) {
            Some(db) => db,
            None => {
                self.show_toast("Database not available");
                return;
            }
        };

        // Disable button and show loading state
        btn.set_sensitive(false);
        btn.set_label("Updating...");

        // Spawn async task
        let view = self.clone();
        let btn_clone = btn.clone();
        glib::spawn_future_local(async move {
            let result = tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    let mut updater = TrackerUpdater::new();
                    match updater.fetch_trackers().await {
                        Ok(trackers) => {
                            // Save to database
                            if let Err(e) = TrackersDb::replace_all(&db, &trackers) {
                                Err(format!("Failed to save trackers: {}", e))
                            } else {
                                Ok(trackers.len())
                            }
                        }
                        Err(e) => Err(format!("Failed to fetch trackers: {}", e)),
                    }
                })
            })
            .await;

            // Re-enable button
            btn_clone.set_sensitive(true);
            btn_clone.set_label("Update Now");

            match result {
                Ok(Ok(count)) => {
                    view.show_toast(&format!("Updated {} trackers", count));
                }
                Ok(Err(e)) => {
                    view.show_toast(&e);
                }
                Err(e) => {
                    view.show_toast(&format!("Update failed: {}", e));
                }
            }
        });
    }

    fn show_toast(&self, message: &str) {
        if let Some(overlay) = self.imp().toast_overlay.get() {
            let toast = adw::Toast::new(message);
            toast.set_timeout(3);
            overlay.add_toast(toast);
        }
    }

    fn save_setting(&self, key: &str, value: &str) {
        let imp = self.imp();
        if let Some(db) = imp.window.borrow().as_ref().and_then(|w| w.db()) {
            if let Err(e) = SettingsDb::set(db, key, value) {
                log::error!("Failed to save setting '{}': {}", key, e);
            }
        }
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

        // Save to database
        self.save_setting("download_path", path);
    }
}
