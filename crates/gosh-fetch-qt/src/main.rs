mod qml;

use std::path::PathBuf;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn resolve_qml_path() -> PathBuf {
    if let Ok(dir) = std::env::var("GOSH_FETCH_QML_DIR") {
        let candidate = PathBuf::from(dir).join("Main.qml");
        if candidate.exists() {
            return candidate;
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let candidate = PathBuf::from(manifest_dir)
            .join("crates")
            .join("gosh-fetch-qt")
            .join("qml")
            .join("Main.qml");
        if candidate.exists() {
            return candidate;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent
                .join("..")
                .join("share")
                .join("gosh-fetch")
                .join("qml")
                .join("Main.qml");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from("qml/Main.qml")
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut app = QGuiApplication::new();

    cxx_qt::qml::register_qml_type::<qml::ffi::AppController>(
        "Gosh.Fetch",
        1,
        0,
        "AppController",
    );

    let mut engine = QQmlApplicationEngine::new();
    let qml_path = resolve_qml_path();
    let qml_url = QUrl::from_local_file(qml_path.to_string_lossy().to_string());
    engine.load(qml_url);

    app.exec();
}
