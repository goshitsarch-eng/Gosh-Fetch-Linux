// Settings page

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Page {
    id: root
    title: "Settings"

    required property var controller

    ScrollView {
        anchors.fill: parent
        anchors.margins: 16

        ColumnLayout {
            width: parent.width
            spacing: 16

            // General section
            Label {
                text: "General"
                font.bold: true
                font.pixelSize: 18
            }

            Frame {
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 12

                    RowLayout {
                        Layout.fillWidth: true

                        Label {
                            text: "Download Location"
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "Browse"
                        }
                    }

                    CheckBox {
                        text: "Enable notifications"
                        checked: true
                    }

                    CheckBox {
                        text: "Close to tray"
                        checked: true
                    }
                }
            }

            // Connection section
            Label {
                text: "Connection"
                font.bold: true
                font.pixelSize: 18
            }

            Frame {
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 12

                    RowLayout {
                        Layout.fillWidth: true

                        Label {
                            text: "Concurrent Downloads"
                        }

                        SpinBox {
                            from: 1
                            to: 20
                            value: 5
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true

                        Label {
                            text: "Connections per Server"
                        }

                        SpinBox {
                            from: 1
                            to: 16
                            value: 8
                        }
                    }
                }
            }

            // BitTorrent section
            Label {
                text: "BitTorrent"
                font.bold: true
                font.pixelSize: 18
            }

            Frame {
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 12

                    CheckBox {
                        text: "Enable DHT"
                        checked: true
                    }

                    CheckBox {
                        text: "Enable PEX"
                        checked: true
                    }

                    CheckBox {
                        text: "Enable LPD"
                        checked: true
                    }

                    CheckBox {
                        text: "Auto-update tracker list"
                        checked: true
                    }
                }
            }
        }
    }
}
