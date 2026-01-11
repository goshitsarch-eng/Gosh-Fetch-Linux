//! Application module - AdwApplication subclass

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::{OnceCell, RefCell};

use crate::window::GoshFetchWindow;
use gosh_fetch_core::{
    init_database, Database, DownloadService, EngineCommand, Settings, SettingsDb, UiMessage,
};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GoshFetchApplication {
        pub db: OnceCell<Database>,
        pub settings: RefCell<Settings>,
        pub cmd_sender: OnceCell<async_channel::Sender<EngineCommand>>,
        pub window: OnceCell<GoshFetchWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GoshFetchApplication {
        const NAME: &'static str = "GoshFetchApplication";
        type Type = super::GoshFetchApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GoshFetchApplication {}

    impl ApplicationImpl for GoshFetchApplication {
        fn activate(&self) {
            let app = self.obj();

            // Get or create the window
            if let Some(window) = self.window.get() {
                window.present();
                return;
            }

            // Initialize database
            let db = match init_database() {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to initialize database: {}", e);
                    app.quit();
                    return;
                }
            };

            // Load settings
            let settings = match SettingsDb::load(&db) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to load settings, using defaults: {}", e);
                    Settings::default()
                }
            };

            // Store database and settings
            let _ = self.db.set(db.clone());
            *self.settings.borrow_mut() = settings.clone();

            // Create channels
            let (ui_sender, ui_receiver) = async_channel::bounded::<UiMessage>(100);
            let (cmd_sender, cmd_receiver) = async_channel::bounded::<EngineCommand>(100);
            let _ = self.cmd_sender.set(cmd_sender.clone());

            // Create download service in a background thread
            let settings_clone = settings.clone();
            let ui_sender_clone = ui_sender.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(async {
                    match DownloadService::new_async(&settings_clone).await {
                        Ok(service) => {
                            service.spawn(ui_sender_clone, cmd_receiver);
                            // Keep thread alive
                            loop {
                                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create download service: {}", e);
                            let _ = ui_sender_clone.send(UiMessage::Error(e.to_string())).await;
                        }
                    }
                });
            });

            // Create the main window
            let window = GoshFetchWindow::new(&*app, db, cmd_sender);
            let _ = self.window.set(window.clone());

            // Set up UI message handler
            let window_weak = window.downgrade();
            glib::spawn_future_local(async move {
                while let Ok(msg) = ui_receiver.recv().await {
                    if let Some(window) = window_weak.upgrade() {
                        window.handle_ui_message(msg);
                    } else {
                        break;
                    }
                }
            });

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            let app = self.obj();

            // Register icon theme for app icons
            if let Some(display) = gtk::gdk::Display::default() {
                let icon_theme = gtk::IconTheme::for_display(&display);
                icon_theme.add_resource_path("/io/github/gosh/Fetch/icons");
            }

            // Set up application actions
            app.setup_actions();

            // Set up keyboard shortcuts
            app.setup_shortcuts();
        }
    }

    impl GtkApplicationImpl for GoshFetchApplication {}
    impl AdwApplicationImpl for GoshFetchApplication {}
}

glib::wrapper! {
    pub struct GoshFetchApplication(ObjectSubclass<imp::GoshFetchApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GoshFetchApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "io.github.gosh.Fetch")
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build()
    }

    fn setup_actions(&self) {
        // Quit action
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| {
                app.quit();
            })
            .build();

        // About action
        let about_action = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about();
            })
            .build();

        self.add_action_entries([quit_action, about_action]);
    }

    fn setup_shortcuts(&self) {
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
        self.set_accels_for_action("win.add-download", &["<Primary>n"]);
        self.set_accels_for_action("win.pause-all", &["<Primary><Shift>p"]);
        self.set_accels_for_action("win.resume-all", &["<Primary><Shift>r"]);
    }

    fn show_about(&self) {
        let window = self.active_window();

        let about = adw::AboutDialog::builder()
            .application_name("Gosh-Fetch")
            .application_icon("io.github.gosh.Fetch")
            .developer_name("Gosh")
            .version("2.0.0")
            .website("https://github.com/goshitsarch-eng/Gosh-Fetch-linux")
            .issue_url("https://github.com/goshitsarch-eng/Gosh-Fetch-linux/issues")
            .license_type(gtk::License::Agpl30)
            .comments("A modern download manager with native Rust engine\n\nFeatures:\n- HTTP/HTTPS segmented downloads\n- BitTorrent and Magnet support\n- DHT, PEX, LPD peer discovery")
            .developers(vec!["Gosh"])
            .build();

        if let Some(window) = window {
            about.present(Some(&window));
        }
    }

    pub fn db(&self) -> Option<&Database> {
        self.imp().db.get()
    }

    pub fn settings(&self) -> Settings {
        self.imp().settings.borrow().clone()
    }

    pub fn update_settings(&self, settings: Settings) {
        *self.imp().settings.borrow_mut() = settings;
    }

    pub fn send_command(&self, cmd: EngineCommand) {
        if let Some(sender) = self.imp().cmd_sender.get() {
            let _ = sender.send_blocking(cmd);
        }
    }
}

impl Default for GoshFetchApplication {
    fn default() -> Self {
        Self::new()
    }
}
