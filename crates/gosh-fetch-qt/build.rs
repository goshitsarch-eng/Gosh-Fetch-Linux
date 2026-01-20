fn main() {
    cxx_qt_build::CxxQtBuilder::new()
        .file("src/qml.rs")
        .build();
}
