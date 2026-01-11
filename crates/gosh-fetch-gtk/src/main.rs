//! Gosh-Fetch GTK - GTK4/libadwaita frontend for Gosh-Fetch download manager

mod application;
mod dialogs;
mod models;
mod tray;
mod views;
mod widgets;
mod window;

use adw::prelude::*;
use gtk::gio;

use application::GoshFetchApplication;

fn main() -> glib::ExitCode {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Gosh-Fetch GTK v2.0.0");

    // Register resources
    gio::resources_register_include!("gosh-fetch.gresource").expect("Failed to register resources");

    // Create and run application
    let app = GoshFetchApplication::new();
    app.run()
}
