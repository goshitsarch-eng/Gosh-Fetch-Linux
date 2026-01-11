// Completed downloads page

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Page {
    id: root
    title: "Completed"

    required property var controller

    // Parse completed downloads JSON from controller
    property var completed: {
        try {
            return JSON.parse(controller.get_completed_json())
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
                    text: controller.completed_count + " completed downloads"
                    opacity: 0.7
                }

                Item { Layout.fillWidth: true }

                Button {
                    text: "Clear History"
                    icon.name: "user-trash"
                    // TODO: Add clear history functionality
                }
            }

            // Empty state
            Label {
                text: "No completed downloads yet."
                visible: completed.length === 0
                opacity: 0.5
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
                Layout.topMargin: 64
            }

            // Completed list from real data
            Repeater {
                model: completed

                delegate: Frame {
                    required property var modelData
                    Layout.fillWidth: true

                    RowLayout {
                        anchors.fill: parent
                        spacing: 12

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Label {
                                text: modelData.name || "Unknown"
                                font.bold: true
                                elide: Text.ElideMiddle
                                Layout.fillWidth: true
                            }

                            Label {
                                text: formatSize(modelData.total_size || 0) + " - " + (modelData.save_path || "")
                                opacity: 0.7
                                font.pixelSize: 12
                                elide: Text.ElideMiddle
                                Layout.fillWidth: true
                            }
                        }

                        ToolButton {
                            icon.name: "folder-open"
                            ToolTip.text: "Open folder"
                            ToolTip.visible: hovered
                            onClicked: controller.open_folder(modelData.save_path)
                        }

                        ToolButton {
                            icon.name: "user-trash"
                            ToolTip.text: "Remove from history"
                            ToolTip.visible: hovered
                            onClicked: controller.remove_download(modelData.gid, false)
                        }
                    }
                }
            }
        }
    }
}
