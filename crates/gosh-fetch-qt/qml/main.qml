// Main QML file for Gosh-Fetch Qt frontend

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import io.github.gosh.Fetch

ApplicationWindow {
    id: root
    visible: true
    width: 1200
    height: 800
    title: "Gosh-Fetch"

    // App controller from Rust
    AppController {
        id: appController
    }

    // Periodic refresh timer
    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: appController.refresh()
    }

    // Navigation drawer
    Drawer {
        id: drawer
        width: 240
        height: root.height

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            // App title
            Label {
                text: "Gosh-Fetch"
                font.pixelSize: 24
                font.bold: true
                Layout.margins: 16
            }

            // Navigation items
            ItemDelegate {
                text: "Downloads"
                icon.name: "folder-download"
                Layout.fillWidth: true
                onClicked: {
                    stackView.replace(downloadsPage)
                    drawer.close()
                }
            }

            ItemDelegate {
                text: "Completed"
                icon.name: "emblem-ok"
                Layout.fillWidth: true
                onClicked: {
                    stackView.replace(completedPage)
                    drawer.close()
                }
            }

            ItemDelegate {
                text: "Settings"
                icon.name: "preferences-system"
                Layout.fillWidth: true
                onClicked: {
                    stackView.replace(settingsPage)
                    drawer.close()
                }
            }

            ItemDelegate {
                text: "About"
                icon.name: "help-about"
                Layout.fillWidth: true
                onClicked: {
                    aboutDialog.open()
                    drawer.close()
                }
            }

            Item { Layout.fillHeight: true }

            // Speed display
            Label {
                text: "↓ " + appController.download_speed + "  ↑ " + appController.upload_speed
                opacity: 0.7
                Layout.margins: 16
            }
        }
    }

    // Header
    header: ToolBar {
        RowLayout {
            anchors.fill: parent

            ToolButton {
                icon.name: "application-menu"
                onClicked: drawer.open()
            }

            Label {
                text: stackView.currentItem ? stackView.currentItem.title : "Downloads"
                elide: Label.ElideRight
                horizontalAlignment: Qt.AlignHCenter
                verticalAlignment: Qt.AlignVCenter
                Layout.fillWidth: true
                font.pixelSize: 18
            }

            ToolButton {
                icon.name: "list-add"
                onClicked: addDialog.open()
            }
        }
    }

    // Main content
    StackView {
        id: stackView
        anchors.fill: parent
        initialItem: downloadsPage
    }

    // Pages
    Component {
        id: downloadsPage
        DownloadsPage {
            controller: appController
        }
    }

    Component {
        id: completedPage
        CompletedPage {
            controller: appController
        }
    }

    Component {
        id: settingsPage
        SettingsPage {
            controller: appController
        }
    }

    // About dialog
    Dialog {
        id: aboutDialog
        title: "About Gosh-Fetch"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok

        ColumnLayout {
            spacing: 12
            width: 350

            Label {
                text: "Gosh-Fetch"
                font.pixelSize: 24
                font.bold: true
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "Version 2.0.0"
                opacity: 0.7
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "A modern download manager with native Rust engine"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
            }

            Label {
                text: "Features:"
                font.bold: true
                Layout.topMargin: 8
            }

            Label {
                text: "• HTTP/HTTPS segmented downloads\n• BitTorrent and Magnet support\n• DHT, PEX, LPD peer discovery"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            Label {
                text: "License: AGPL-3.0"
                opacity: 0.7
                Layout.topMargin: 8
            }

            Label {
                text: '<a href="https://github.com/goshitsarch-eng/Gosh-Fetch-linux">GitHub</a>'
                textFormat: Text.RichText
                onLinkActivated: Qt.openUrlExternally(link)
                Layout.alignment: Qt.AlignHCenter
            }
        }
    }

    // Add download dialog with tabs
    Dialog {
        id: addDialog
        title: "Add Download"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Ok | Dialog.Cancel

        ColumnLayout {
            spacing: 16
            width: 450

            TabBar {
                id: addTabBar
                Layout.fillWidth: true

                TabButton {
                    text: "URL"
                }
                TabButton {
                    text: "Magnet"
                }
                TabButton {
                    text: "Torrent File"
                }
            }

            StackLayout {
                currentIndex: addTabBar.currentIndex
                Layout.fillWidth: true

                // URL tab
                ColumnLayout {
                    spacing: 8

                    TextField {
                        id: urlField
                        placeholderText: "https://example.com/file.zip"
                        Layout.fillWidth: true
                    }

                    Label {
                        text: "Supports HTTP, HTTPS, FTP, and magnet links"
                        opacity: 0.7
                        font.pixelSize: 12
                    }
                }

                // Magnet tab
                ColumnLayout {
                    spacing: 8

                    TextField {
                        id: magnetField
                        placeholderText: "magnet:?xt=urn:btih:..."
                        Layout.fillWidth: true
                    }

                    Label {
                        text: "Paste your magnet link here"
                        opacity: 0.7
                        font.pixelSize: 12
                    }
                }

                // Torrent file tab
                ColumnLayout {
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true

                        TextField {
                            id: torrentPathField
                            placeholderText: "No file selected"
                            readOnly: true
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "Browse..."
                            onClicked: torrentFileDialog.open()
                        }
                    }

                    Label {
                        text: "Select a .torrent file from your computer"
                        opacity: 0.7
                        font.pixelSize: 12
                    }
                }
            }
        }

        onAccepted: {
            switch (addTabBar.currentIndex) {
                case 0: // URL
                    if (urlField.text.length > 0) {
                        appController.add_download(urlField.text)
                        urlField.text = ""
                    }
                    break
                case 1: // Magnet
                    if (magnetField.text.length > 0) {
                        appController.add_download(magnetField.text)
                        magnetField.text = ""
                    }
                    break
                case 2: // Torrent file
                    if (torrentPathField.text.length > 0) {
                        appController.add_torrent_file(torrentPathField.text)
                        torrentPathField.text = ""
                    }
                    break
            }
        }

        onOpened: {
            addTabBar.currentIndex = 0
            urlField.text = ""
            magnetField.text = ""
            torrentPathField.text = ""
        }
    }

    // File dialog for torrent files
    FileDialog {
        id: torrentFileDialog
        title: "Select Torrent File"
        nameFilters: ["Torrent files (*.torrent)"]
        onAccepted: {
            torrentPathField.text = selectedFile.toString().replace("file://", "")
        }
    }
}
