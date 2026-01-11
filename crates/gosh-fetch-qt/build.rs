//! Build script for Qt6 frontend
//!
//! This uses cxx-qt-build to compile Qt resources and generate bindings.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .qml_module(QmlModule {
            uri: "io.github.gosh.Fetch",
            rust_files: &["src/bridge.rs"],
            qml_files: &[
                "qml/main.qml",
                "qml/DownloadsPage.qml",
                "qml/CompletedPage.qml",
                "qml/SettingsPage.qml",
            ],
            ..Default::default()
        })
        .build();
}
