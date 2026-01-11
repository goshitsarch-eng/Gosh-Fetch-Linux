//! Gosh-Fetch Qt - Qt6/QML frontend for Gosh-Fetch download manager
//!
//! This crate provides a native Qt6/QML experience using CXX-Qt for Rust/Qt interop.

mod bridge;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use gosh_fetch_core::{init_database, DownloadService, EngineCommand, SettingsDb, UiMessage};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Gosh-Fetch Qt v2.0.0");

    // Initialize database
    let db = match init_database() {
        Ok(db) => {
            log::info!("Database initialized successfully");
            Some(db)
        }
        Err(e) => {
            log::error!("Failed to initialize database: {}", e);
            None
        }
    };

    // Load settings
    let settings = db
        .as_ref()
        .and_then(|db| SettingsDb::load(db).ok())
        .unwrap_or_default();

    // Create channels for engine communication
    let (ui_sender, ui_receiver) = async_channel::bounded::<UiMessage>(100);
    let (cmd_sender, cmd_receiver) = async_channel::bounded::<EngineCommand>(100);

    // Store command sender for bridge to use
    bridge::set_command_sender(cmd_sender.clone());

    // Spawn download service in background thread
    let settings_clone = settings.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            match DownloadService::new_async(&settings_clone).await {
                Ok(service) => {
                    log::info!("Download service started");
                    service.spawn(ui_sender.clone(), cmd_receiver);
                    // Keep thread alive
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    }
                }
                Err(e) => {
                    log::error!("Failed to create download service: {}", e);
                }
            }
        });
    });

    // Spawn UI message receiver thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            while let Ok(msg) = ui_receiver.recv().await {
                bridge::handle_ui_message(msg);
            }
        });
    });

    // Create Qt application
    let mut app = QGuiApplication::new();

    // Create QML engine
    let mut engine = QQmlApplicationEngine::new();

    // Load main QML file from resources
    // The path follows: qrc:/qt/qml/<uri_as_path>/qml/<file>
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/io/github/gosh/Fetch/qml/main.qml"));
    }

    // Run the application
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
