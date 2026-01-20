import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1
import Gosh.Fetch 1.0

ApplicationWindow {
    id: root
    width: 1200
    height: 800
    visible: true
    title: "Gosh-Fetch"
    color: "#0f1115"

    AppController {
        id: controller
    }

    property var settings: ({})
    property var stats: ({ download_speed: 0, upload_speed: 0, num_active: 0, num_waiting: 0, num_stopped: 0 })

    ListModel { id: downloadsModel }
    ListModel { id: completedModel }

    function formatBytes(bytes) {
        var kb = 1024; var mb = kb * 1024; var gb = mb * 1024; var tb = gb * 1024;
        if (bytes >= tb) return (bytes / tb).toFixed(2) + " TB";
        if (bytes >= gb) return (bytes / gb).toFixed(2) + " GB";
        if (bytes >= mb) return (bytes / mb).toFixed(2) + " MB";
        if (bytes >= kb) return (bytes / kb).toFixed(2) + " KB";
        return bytes + " B";
    }

    function formatSpeed(bytes) {
        if (bytes === 0) return "0 B/s";
        return formatBytes(bytes) + "/s";
    }

    function upsertDownload(model, download) {
        for (var i = 0; i < model.count; ++i) {
            if (model.get(i).gid === download.gid) {
                model.set(i, download);
                return;
            }
        }
        model.append(download);
    }

    function removeByGid(model, gid) {
        for (var i = 0; i < model.count; ++i) {
            if (model.get(i).gid === gid) {
                model.remove(i);
                return;
            }
        }
    }

    Component.onCompleted: {
        controller.initialize();
        controller.refresh_downloads();
        controller.refresh_stats();
        controller.load_completed();
        var raw = controller.get_settings_json();
        if (raw.length > 0) {
            settings = JSON.parse(raw);
        }
        uaCombo.currentIndex = controller.get_user_agent_index(settings.user_agent || "");
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            controller.poll();
            controller.refresh_downloads();
            controller.refresh_stats();
        }
    }

    Connections {
        target: controller

        function onDownload_added(json) {
            var download = JSON.parse(json);
            if (download.status === "complete") {
                upsertDownload(completedModel, download);
                removeByGid(downloadsModel, download.gid);
                return;
            }
            upsertDownload(downloadsModel, download);
        }

        function onDownload_updated(gid, json) {
            var download = JSON.parse(json);
            if (download.status === "complete") {
                upsertDownload(completedModel, download);
                removeByGid(downloadsModel, gid);
                return;
            }
            upsertDownload(downloadsModel, download);
        }

        function onDownload_removed(gid) {
            removeByGid(downloadsModel, gid);
            removeByGid(completedModel, gid);
        }

        function onDownload_completed(json) {
            var download = JSON.parse(json);
            upsertDownload(completedModel, download);
            removeByGid(downloadsModel, download.gid);
        }

        function onStats_updated(json) {
            stats = JSON.parse(json);
        }

        function onDownloads_list(json) {
            downloadsModel.clear();
            var list = JSON.parse(json);
            for (var i = 0; i < list.length; ++i) {
                if (list[i].status !== "complete") {
                    downloadsModel.append(list[i]);
                }
            }
        }

        function onCompleted_list(json) {
            completedModel.clear();
            var list = JSON.parse(json);
            for (var i = 0; i < list.length; ++i) {
                completedModel.append(list[i]);
            }
        }

        function onError(message) {
            toastLabel.text = message;
            toast.open();
        }

        function onToast(message) {
            toastLabel.text = message;
            toast.open();
        }
    }

    SystemTrayIcon {
        id: tray
        visible: settings.close_to_tray === true
        icon.source: controller.get_icon_path()
        tooltip: "Gosh-Fetch"
        menu: Menu {
            MenuItem { text: "Show"; onTriggered: root.show() }
            MenuItem { text: "Hide"; onTriggered: root.hide() }
            MenuSeparator { }
            MenuItem { text: "Pause All"; onTriggered: controller.pause_all() }
            MenuItem { text: "Resume All"; onTriggered: controller.resume_all() }
            MenuSeparator { }
            MenuItem { text: "Quit"; onTriggered: Qt.quit() }
        }
        onActivated: root.show()
    }

    onClosing: function(close) {
        if (settings.close_to_tray === true) {
            close.accepted = false;
            root.hide();
        }
    }

    Dialog {
        id: toast
        modal: false
        focus: false
        x: (root.width - width) / 2
        y: root.height - height - 24
        background: Rectangle { color: "#22262f"; radius: 10; border.color: "#3b3f46" }
        contentItem: Text {
            id: toastLabel
            color: "#f2f2f2"
            font.pixelSize: 14
            padding: 12
        }
        closePolicy: Popup.CloseOnPressOutside
        Timer { interval: 3000; running: toast.visible; onTriggered: toast.close() }
    }

    header: ToolBar {
        height: 58
        background: Rectangle {
            color: "#141821"
            border.color: "#2a2f3a"
        }
        RowLayout {
            anchors.fill: parent
            anchors.margins: 12
            spacing: 14

            Text {
                text: "Gosh-Fetch"
                color: "#f2f2f2"
                font.pixelSize: 20
                font.family: "IBM Plex Sans"
            }

            Item { Layout.fillWidth: true }

            Rectangle {
                radius: 16
                color: "#1d2430"
                border.color: "#2a3443"
                height: 32
                Layout.alignment: Qt.AlignVCenter
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 6
                    spacing: 8
                    Text { text: "↓ " + formatSpeed(stats.download_speed); color: "#7de2d1"; font.pixelSize: 12 }
                    Text { text: "↑ " + formatSpeed(stats.upload_speed); color: "#f7b267"; font.pixelSize: 12 }
                }
            }

            Button {
                text: "Add"
                onClicked: addDialog.open()
            }
            Button {
                text: "Pause All"
                onClicked: controller.pause_all()
            }
            Button {
                text: "Resume All"
                onClicked: controller.resume_all()
            }
        }
    }

    Shortcut { sequence: "Ctrl+N"; onActivated: addDialog.open() }
    Shortcut { sequence: "Ctrl+Shift+P"; onActivated: controller.pause_all() }
    Shortcut { sequence: "Ctrl+Shift+R"; onActivated: controller.resume_all() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        TabBar {
            id: navTabs
            Layout.fillWidth: true
            currentIndex: 0
            background: Rectangle { color: "#11141b"; radius: 10; border.color: "#273041" }
            TabButton { text: "Downloads" }
            TabButton { text: "Completed" }
            TabButton { text: "Settings" }
        }

        StackLayout {
            id: pages
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: navTabs.currentIndex

            // Downloads page
            Rectangle {
                color: "#11141b"
                radius: 12
                border.color: "#273041"
                anchors.fill: parent

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 10

                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: downloadsModel.count + " downloads"
                            color: "#9aa3b2"
                            font.pixelSize: 12
                        }

                        Item { Layout.fillWidth: true }

                        ComboBox {
                            id: filterBox
                            model: ["All", "Active", "Paused", "Error"]
                            currentIndex: 0
                        }
                    }

                    ListView {
                        id: downloadsList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: 10
                        clip: true
                        model: downloadsModel
                        delegate: Rectangle {
                            width: downloadsList.width
                            height: 132
                            radius: 12
                            color: "#161b24"
                            border.color: "#2a3443"
                            visible: {
                                if (filterBox.currentIndex === 0) return true;
                                if (filterBox.currentIndex === 1) return model.status === "active" || model.status === "waiting";
                                if (filterBox.currentIndex === 2) return model.status === "paused";
                                if (filterBox.currentIndex === 3) return model.status === "error";
                                return true;
                            }

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 6

                                RowLayout {
                                    Layout.fillWidth: true
                                    Text { text: model.name; color: "#f2f2f2"; font.pixelSize: 16; elide: Text.ElideRight; Layout.fillWidth: true }
                                    Text { text: model.status; color: "#8c96a6"; font.pixelSize: 12 }
                                }

                                ProgressBar {
                                    from: 0
                                    to: 1
                                    value: model.total_size > 0 ? model.completed_size / model.total_size : 0
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    Text { text: formatBytes(model.completed_size) + " / " + formatBytes(model.total_size); color: "#8c96a6"; font.pixelSize: 12 }
                                    Item { Layout.fillWidth: true }
                                    Text { text: "↓ " + formatSpeed(model.download_speed); color: "#7de2d1"; font.pixelSize: 12 }
                                    Text { text: "↑ " + formatSpeed(model.upload_speed); color: "#f7b267"; font.pixelSize: 12 }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    Button {
                                        text: model.status === "paused" ? "Resume" : "Pause"
                                        onClicked: {
                                            if (model.status === "paused") {
                                                controller.resume_download(model.gid)
                                            } else {
                                                controller.pause_download(model.gid)
                                            }
                                        }
                                    }
                                    Button {
                                        text: "Remove"
                                        onClicked: controller.remove_download(model.gid, settings.delete_files_on_remove === true)
                                    }
                                    Button {
                                        text: "Open"
                                        onClicked: controller.open_path(model.save_path)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Completed page
            Rectangle {
                color: "#11141b"
                radius: 12
                border.color: "#273041"
                anchors.fill: parent

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 10

                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            text: completedModel.count + " completed"
                            color: "#9aa3b2"
                            font.pixelSize: 12
                        }
                        Item { Layout.fillWidth: true }
                        Button {
                            text: "Clear History"
                            onClicked: {
                                controller.clear_completed();
                                controller.load_completed();
                            }
                        }
                    }

                    ListView {
                        id: completedList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: 10
                        clip: true
                        model: completedModel
                        delegate: Rectangle {
                            width: completedList.width
                            height: 110
                            radius: 12
                            color: "#161b24"
                            border.color: "#2a3443"
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 6
                                RowLayout {
                                    Layout.fillWidth: true
                                    Text { text: model.name; color: "#f2f2f2"; font.pixelSize: 16; elide: Text.ElideRight; Layout.fillWidth: true }
                                    Text { text: "completed"; color: "#8c96a6"; font.pixelSize: 12 }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Text { text: formatBytes(model.total_size); color: "#8c96a6"; font.pixelSize: 12 }
                                    Item { Layout.fillWidth: true }
                                    Text { text: model.completed_at || ""; color: "#6f7a8c"; font.pixelSize: 12 }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Button {
                                        text: "Open"
                                        onClicked: controller.open_path(model.save_path)
                                    }
                                    Button {
                                        text: "Remove"
                                        onClicked: controller.remove_download(model.gid, settings.delete_files_on_remove === true)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Settings page
            Rectangle {
                color: "#11141b"
                radius: 12
                border.color: "#273041"
                anchors.fill: parent

                ScrollView {
                    anchors.fill: parent
                    contentWidth: parent.width

                    ColumnLayout {
                        width: parent.width
                        spacing: 18
                        padding: 20

                        GroupBox {
                            title: "General"
                            Layout.fillWidth: true
                            ColumnLayout {
                                spacing: 8
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Download Path"; Layout.preferredWidth: 160 }
                                    TextField {
                                        Layout.fillWidth: true
                                        text: settings.download_path || ""
                                        onEditingFinished: {
                                            controller.set_setting("download_path", text)
                                            settings.download_path = text
                                        }
                                    }
                                    Button { text: "Browse"; onClicked: downloadPathDialog.open() }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Notifications"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.enable_notifications === true
                                        onToggled: {
                                            controller.set_setting("enable_notifications", checked ? "true" : "false")
                                            settings.enable_notifications = checked
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Close to Tray"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.close_to_tray === true
                                        onToggled: {
                                            controller.set_setting("close_to_tray", checked ? "true" : "false")
                                            settings.close_to_tray = checked
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Delete Files On Remove"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.delete_files_on_remove === true
                                        onToggled: {
                                            controller.set_setting("delete_files_on_remove", checked ? "true" : "false")
                                            settings.delete_files_on_remove = checked
                                        }
                                    }
                                }
                            }
                        }

                        GroupBox {
                            title: "Connection"
                            Layout.fillWidth: true
                            ColumnLayout {
                                spacing: 8
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Max Concurrent"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 1; to: 20
                                        value: settings.max_concurrent_downloads || 5
                                        onValueModified: {
                                            controller.set_setting("max_concurrent_downloads", value.toString())
                                            settings.max_concurrent_downloads = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Connections / Server"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 1; to: 16
                                        value: settings.max_connections_per_server || 16
                                        onValueModified: {
                                            controller.set_setting("max_connections_per_server", value.toString())
                                            settings.max_connections_per_server = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Split Count"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 1; to: 64
                                        value: settings.split_count || 16
                                        onValueModified: {
                                            controller.set_setting("split_count", value.toString())
                                            settings.split_count = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Min Segment Size (KB)"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 256; to: 10240
                                        value: settings.min_segment_size || 1024
                                        onValueModified: {
                                            controller.set_setting("min_segment_size", value.toString())
                                            settings.min_segment_size = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Download Limit (KB/s)"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 0; to: 100000
                                        value: settings.download_speed_limit ? Math.round(settings.download_speed_limit / 1024) : 0
                                        onValueModified: {
                                            var bytes = value * 1024;
                                            controller.set_setting("download_speed_limit", bytes.toString())
                                            settings.download_speed_limit = bytes
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Upload Limit (KB/s)"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 0; to: 100000
                                        value: settings.upload_speed_limit ? Math.round(settings.upload_speed_limit / 1024) : 0
                                        onValueModified: {
                                            var bytes = value * 1024;
                                            controller.set_setting("upload_speed_limit", bytes.toString())
                                            settings.upload_speed_limit = bytes
                                        }
                                    }
                                }
                            }
                        }

                        GroupBox {
                            title: "User Agent"
                            Layout.fillWidth: true
                            ColumnLayout {
                                spacing: 8
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Preset"; Layout.preferredWidth: 160 }
                                    ComboBox {
                                        id: uaCombo
                                        model: JSON.parse(controller.get_user_agent_presets_json())
                                        onActivated: {
                                            var value = controller.get_user_agent_value(currentIndex)
                                            controller.set_setting("user_agent", value)
                                            settings.user_agent = value
                                        }
                                    }
                                }
                            }
                        }

                        GroupBox {
                            title: "Proxy"
                            Layout.fillWidth: true
                            ColumnLayout {
                                spacing: 8
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Enable Proxy"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.proxy_enabled === true
                                        onToggled: {
                                            controller.set_setting("proxy_enabled", checked ? "true" : "false")
                                            settings.proxy_enabled = checked
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Type"; Layout.preferredWidth: 160 }
                                    ComboBox {
                                        model: ["http", "https", "socks5"]
                                        currentIndex: settings.proxy_type === "https" ? 1 : (settings.proxy_type === "socks5" ? 2 : 0)
                                        onActivated: {
                                            var value = model[currentIndex]
                                            controller.set_setting("proxy_type", value)
                                            settings.proxy_type = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Proxy URL"; Layout.preferredWidth: 160 }
                                    TextField {
                                        Layout.fillWidth: true
                                        text: settings.proxy_url || ""
                                        onEditingFinished: {
                                            controller.set_setting("proxy_url", text)
                                            settings.proxy_url = text
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Proxy User"; Layout.preferredWidth: 160 }
                                    TextField {
                                        Layout.fillWidth: true
                                        text: settings.proxy_user || ""
                                        onEditingFinished: controller.set_setting("proxy_user", text)
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Proxy Pass"; Layout.preferredWidth: 160 }
                                    TextField {
                                        Layout.fillWidth: true
                                        echoMode: TextInput.Password
                                        text: settings.proxy_pass || ""
                                        onEditingFinished: controller.set_setting("proxy_pass", text)
                                    }
                                }
                            }
                        }

                        GroupBox {
                            title: "BitTorrent"
                            Layout.fillWidth: true
                            ColumnLayout {
                                spacing: 8
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Enable DHT"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.bt_enable_dht === true
                                        onToggled: {
                                            controller.set_setting("bt_enable_dht", checked ? "true" : "false")
                                            settings.bt_enable_dht = checked
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Enable PEX"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.bt_enable_pex === true
                                        onToggled: {
                                            controller.set_setting("bt_enable_pex", checked ? "true" : "false")
                                            settings.bt_enable_pex = checked
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Enable LPD"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.bt_enable_lpd === true
                                        onToggled: {
                                            controller.set_setting("bt_enable_lpd", checked ? "true" : "false")
                                            settings.bt_enable_lpd = checked
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Max Peers"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 10; to: 300
                                        value: settings.bt_max_peers || 55
                                        onValueModified: {
                                            controller.set_setting("bt_max_peers", value.toString())
                                            settings.bt_max_peers = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Seed Ratio"; Layout.preferredWidth: 160 }
                                    SpinBox {
                                        from: 0; to: 10
                                        value: settings.bt_seed_ratio || 1.0
                                        stepSize: 0.1
                                        onValueModified: {
                                            controller.set_setting("bt_seed_ratio", value.toString())
                                            settings.bt_seed_ratio = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Preallocation"; Layout.preferredWidth: 160 }
                                    ComboBox {
                                        model: ["none", "sparse", "full"]
                                        currentIndex: settings.bt_preallocation === "full" ? 2 : (settings.bt_preallocation === "none" ? 0 : 1)
                                        onActivated: {
                                            var value = model[currentIndex]
                                            controller.set_setting("bt_preallocation", value)
                                            settings.bt_preallocation = value
                                        }
                                    }
                                }
                                RowLayout {
                                    Layout.fillWidth: true
                                    Label { text: "Auto Update Trackers"; Layout.preferredWidth: 160 }
                                    Switch {
                                        checked: settings.auto_update_trackers === true
                                        onToggled: {
                                            controller.set_setting("auto_update_trackers", checked ? "true" : "false")
                                            settings.auto_update_trackers = checked
                                        }
                                    }
                                    Button {
                                        text: "Update Now"
                                        onClicked: controller.update_trackers()
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    FolderDialog {
        id: downloadPathDialog
        title: "Select Download Location"
        folder: StandardPaths.writableLocation(StandardPaths.DownloadLocation)
        onAccepted: {
            var path = downloadPathDialog.currentFolder.toString().replace("file://", "");
            controller.set_setting("download_path", path);
            settings.download_path = path;
        }
    }

    Dialog {
        id: addDialog
        width: 640
        height: 600
        modal: true
        title: "Add Download"
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: {
            var options = {
                dir: downloadDirField.text.length > 0 ? downloadDirField.text : undefined,
                out: filenameField.text.length > 0 ? filenameField.text : undefined,
                max_connection_per_server: connectionsField.text.length > 0 ? connectionsField.text : undefined,
                user_agent: userAgentField.text.length > 0 ? userAgentField.text : undefined,
                referer: refererField.text.length > 0 ? refererField.text : undefined,
                header: headersField.text.length > 0 ? headersField.text.split("\n") : undefined,
                cookies: cookiesField.text.length > 0 ? cookiesField.text : undefined,
                checksum_type: checksumTypeCombo.currentText !== "None" ? checksumTypeCombo.currentText.toLowerCase() : undefined,
                checksum_value: checksumValueField.text.length > 0 ? checksumValueField.text : undefined,
                mirror_urls: mirrorsField.text.length > 0 ? mirrorsField.text.split("\n") : undefined,
                priority: priorityCombo.currentText.toLowerCase(),
                max_download_limit: speedLimitField.text.length > 0 ? speedLimitField.text : undefined,
                max_upload_limit: uploadLimitField.text.length > 0 ? uploadLimitField.text : undefined,
                sequential: sequentialSwitch.checked,
                select_file: selectedFilesField.text.length > 0 ? selectedFilesField.text : undefined,
                seed_ratio: seedRatioField.text.length > 0 ? seedRatioField.text : undefined
            };
            if (scheduleSwitch.checked) {
                options.scheduled_start = Math.floor(scheduleTime.dateTime.getTime() / 1000);
            }

            var optionsJson = JSON.stringify(options);

            if (addTabs.currentIndex === 0) {
                controller.add_download(urlField.text, optionsJson)
            } else if (addTabs.currentIndex === 1) {
                controller.add_magnet(magnetField.text, optionsJson)
            } else {
                controller.add_torrent(torrentPathField.text, optionsJson)
            }
        }

        contentItem: ColumnLayout {
            anchors.fill: parent
            spacing: 12

            TabBar {
                id: addTabs
                Layout.fillWidth: true
                TabButton { text: "URL" }
                TabButton { text: "Magnet" }
                TabButton { text: "Torrent" }
            }

            StackLayout {
                Layout.fillWidth: true
                currentIndex: addTabs.currentIndex
                Item {
                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 8
                        TextField { id: urlField; placeholderText: "https://example.com/file.zip"; Layout.fillWidth: true }
                    }
                }
                Item {
                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 8
                        TextArea { id: magnetField; placeholderText: "magnet:?xt=urn:btih:..."; Layout.fillWidth: true; Layout.fillHeight: true }
                    }
                }
                Item {
                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 8
                        RowLayout {
                            Layout.fillWidth: true
                            TextField { id: torrentPathField; Layout.fillWidth: true; placeholderText: "Select torrent file" }
                            Button { text: "Browse"; onClicked: torrentDialog.open() }
                        }
                    }
                }
            }

            GroupBox {
                title: "Advanced Options"
                Layout.fillWidth: true
                ColumnLayout {
                    spacing: 8
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Save Dir"; Layout.preferredWidth: 120 }
                        TextField { id: downloadDirField; Layout.fillWidth: true; text: settings.download_path || "" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Filename"; Layout.preferredWidth: 120 }
                        TextField { id: filenameField; Layout.fillWidth: true }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Connections"; Layout.preferredWidth: 120 }
                        TextField { id: connectionsField; Layout.fillWidth: true; placeholderText: "16" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "User Agent"; Layout.preferredWidth: 120 }
                        TextField { id: userAgentField; Layout.fillWidth: true; text: settings.user_agent || "" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Referer"; Layout.preferredWidth: 120 }
                        TextField { id: refererField; Layout.fillWidth: true }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Headers (one per line)"; Layout.preferredWidth: 120 }
                        TextArea { id: headersField; Layout.fillWidth: true; Layout.preferredHeight: 60 }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Cookies"; Layout.preferredWidth: 120 }
                        TextField { id: cookiesField; Layout.fillWidth: true }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Checksum"; Layout.preferredWidth: 120 }
                        ComboBox { id: checksumTypeCombo; model: ["None", "MD5", "SHA256"] }
                        TextField { id: checksumValueField; Layout.fillWidth: true }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Mirrors (one per line)"; Layout.preferredWidth: 120 }
                        TextArea { id: mirrorsField; Layout.fillWidth: true; Layout.preferredHeight: 60 }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Priority"; Layout.preferredWidth: 120 }
                        ComboBox { id: priorityCombo; model: ["Normal", "Low", "High", "Critical"]; currentIndex: 0 }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Selected Files"; Layout.preferredWidth: 120 }
                        TextField { id: selectedFilesField; Layout.fillWidth: true; placeholderText: "e.g. 0,2,5" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Seed Ratio"; Layout.preferredWidth: 120 }
                        TextField { id: seedRatioField; Layout.fillWidth: true; placeholderText: "e.g. 1.0" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Download Limit"; Layout.preferredWidth: 120 }
                        TextField { id: speedLimitField; Layout.fillWidth: true; placeholderText: "e.g. 5M" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Upload Limit"; Layout.preferredWidth: 120 }
                        TextField { id: uploadLimitField; Layout.fillWidth: true; placeholderText: "e.g. 2M" }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Sequential"; Layout.preferredWidth: 120 }
                        Switch { id: sequentialSwitch }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Schedule"; Layout.preferredWidth: 120 }
                        Switch { id: scheduleSwitch }
                        DateTimeEdit { id: scheduleTime; enabled: scheduleSwitch.checked }
                    }
                }
            }
        }
    }

    FileDialog {
        id: torrentDialog
        title: "Select Torrent File"
        nameFilters: ["Torrent files (*.torrent)"]
        onAccepted: {
            var path = torrentDialog.currentFile.toString().replace("file://", "");
            torrentPathField.text = path;
        }
    }
}
