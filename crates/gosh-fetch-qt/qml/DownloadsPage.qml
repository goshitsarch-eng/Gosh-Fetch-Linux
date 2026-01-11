// Downloads page - shows active downloads

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Page {
    id: root
    title: "Downloads"

    required property var controller

    // Parse downloads JSON from controller
    property var downloads: {
        try {
            return JSON.parse(controller.get_downloads_json())
        } catch (e) {
            return []
        }
    }

    function formatSize(bytes) {
        if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(2) + " GB"
        if (bytes >= 1048576) return (bytes / 1048576).toFixed(2) + " MB"
        if (bytes >= 1024) return (bytes / 1024).toFixed(2) + " KB"
        return bytes + " B"
    }

    function formatSpeed(bytesPerSec) {
        return formatSize(bytesPerSec) + "/s"
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: 16

        ColumnLayout {
            width: parent.width
            spacing: 8

            // Header
            RowLayout {
                Layout.fillWidth: true

                Label {
                    text: controller.active_count + " active downloads"
                    opacity: 0.7
                }

                Item { Layout.fillWidth: true }

                Button {
                    text: "Pause All"
                    icon.name: "media-playback-pause"
                    onClicked: controller.pause_all()
                }

                Button {
                    text: "Resume All"
                    icon.name: "media-playback-start"
                    onClicked: controller.resume_all()
                }
            }

            // Empty state
            Label {
                text: "No active downloads.\nClick + to add a download."
                visible: downloads.length === 0
                opacity: 0.5
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
                Layout.topMargin: 64
            }

            // Downloads list from real data
            Repeater {
                model: downloads

                delegate: Frame {
                    required property var modelData
                    Layout.fillWidth: true

                    property real progress: modelData.total_size > 0
                        ? modelData.completed_size / modelData.total_size
                        : 0
                    property bool isPaused: modelData.status === "Paused"

                    RowLayout {
                        anchors.fill: parent
                        spacing: 12

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Label {
                                text: modelData.name || "Unknown"
                                font.bold: true
                                elide: Text.ElideMiddle
                                Layout.fillWidth: true
                            }

                            ProgressBar {
                                Layout.fillWidth: true
                                value: progress
                            }

                            Label {
                                text: {
                                    var pct = (progress * 100).toFixed(1) + "%"
                                    var sizes = formatSize(modelData.completed_size) + " / " + formatSize(modelData.total_size)
                                    var speed = formatSpeed(modelData.download_speed || 0)
                                    return pct + " - " + sizes + " - " + speed
                                }
                                opacity: 0.7
                                font.pixelSize: 12
                            }
                        }

                        ToolButton {
                            icon.name: isPaused ? "media-playback-start" : "media-playback-pause"
                            onClicked: {
                                if (isPaused) {
                                    controller.resume_download(modelData.gid)
                                } else {
                                    controller.pause_download(modelData.gid)
                                }
                            }
                        }

                        ToolButton {
                            icon.name: "folder-open"
                            onClicked: controller.open_folder(modelData.save_path)
                        }

                        ToolButton {
                            icon.name: "user-trash"
                            onClicked: controller.remove_download(modelData.gid, false)
                        }
                    }
                }
            }
        }
    }
}
