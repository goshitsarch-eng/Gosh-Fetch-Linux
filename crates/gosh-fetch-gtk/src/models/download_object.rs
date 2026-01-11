//! DownloadObject - GObject wrapper for Download data

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use gosh_fetch_core::{Download, DownloadState, DownloadType};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct DownloadObject {
        pub id: Cell<i64>,
        pub gid: RefCell<String>,
        pub name: RefCell<String>,
        pub url: RefCell<Option<String>>,
        pub download_type: Cell<u32>,
        pub status: Cell<u32>,
        pub total_size: Cell<u64>,
        pub completed_size: Cell<u64>,
        pub download_speed: Cell<u64>,
        pub upload_speed: Cell<u64>,
        pub save_path: RefCell<String>,
        pub error_message: RefCell<Option<String>>,
        pub connections: Cell<u32>,
        pub seeders: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DownloadObject {
        const NAME: &'static str = "DownloadObject";
        type Type = super::DownloadObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DownloadObject {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").build(),
                    glib::ParamSpecString::builder("gid").build(),
                    glib::ParamSpecString::builder("name").build(),
                    glib::ParamSpecString::builder("url").build(),
                    glib::ParamSpecUInt::builder("download-type").build(),
                    glib::ParamSpecUInt::builder("status").build(),
                    glib::ParamSpecUInt64::builder("total-size").build(),
                    glib::ParamSpecUInt64::builder("completed-size").build(),
                    glib::ParamSpecUInt64::builder("download-speed").build(),
                    glib::ParamSpecUInt64::builder("upload-speed").build(),
                    glib::ParamSpecString::builder("save-path").build(),
                    glib::ParamSpecString::builder("error-message").build(),
                    glib::ParamSpecUInt::builder("connections").build(),
                    glib::ParamSpecUInt::builder("seeders").build(),
                    // Computed properties
                    glib::ParamSpecDouble::builder("progress")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("status-text")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "id" => self.id.set(value.get().unwrap()),
                "gid" => *self.gid.borrow_mut() = value.get().unwrap(),
                "name" => *self.name.borrow_mut() = value.get().unwrap(),
                "url" => *self.url.borrow_mut() = value.get().ok(),
                "download-type" => self.download_type.set(value.get().unwrap()),
                "status" => self.status.set(value.get().unwrap()),
                "total-size" => self.total_size.set(value.get().unwrap()),
                "completed-size" => self.completed_size.set(value.get().unwrap()),
                "download-speed" => self.download_speed.set(value.get().unwrap()),
                "upload-speed" => self.upload_speed.set(value.get().unwrap()),
                "save-path" => *self.save_path.borrow_mut() = value.get().unwrap(),
                "error-message" => *self.error_message.borrow_mut() = value.get().ok(),
                "connections" => self.connections.set(value.get().unwrap()),
                "seeders" => self.seeders.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "gid" => self.gid.borrow().to_value(),
                "name" => self.name.borrow().to_value(),
                "url" => self.url.borrow().to_value(),
                "download-type" => self.download_type.get().to_value(),
                "status" => self.status.get().to_value(),
                "total-size" => self.total_size.get().to_value(),
                "completed-size" => self.completed_size.get().to_value(),
                "download-speed" => self.download_speed.get().to_value(),
                "upload-speed" => self.upload_speed.get().to_value(),
                "save-path" => self.save_path.borrow().to_value(),
                "error-message" => self.error_message.borrow().to_value(),
                "connections" => self.connections.get().to_value(),
                "seeders" => self.seeders.get().to_value(),
                "progress" => {
                    let total = self.total_size.get();
                    let completed = self.completed_size.get();
                    if total == 0 {
                        0.0f64.to_value()
                    } else {
                        (completed as f64 / total as f64).to_value()
                    }
                }
                "status-text" => {
                    let status = self.status.get();
                    let speed = self.download_speed.get();
                    get_status_text(status, speed).to_value()
                }
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct DownloadObject(ObjectSubclass<imp::DownloadObject>);
}

impl DownloadObject {
    pub fn new(download: &Download) -> Self {
        let obj: Self = glib::Object::new();
        obj.update(download);
        obj
    }

    pub fn update(&self, download: &Download) {
        let imp = self.imp();
        imp.id.set(download.id);
        *imp.gid.borrow_mut() = download.gid.clone();
        *imp.name.borrow_mut() = download.name.clone();
        *imp.url.borrow_mut() = download.url.clone();
        imp.download_type
            .set(download_type_to_u32(download.download_type));
        imp.status.set(status_to_u32(download.status));
        imp.total_size.set(download.total_size);
        imp.completed_size.set(download.completed_size);
        imp.download_speed.set(download.download_speed);
        imp.upload_speed.set(download.upload_speed);
        *imp.save_path.borrow_mut() = download.save_path.clone();
        *imp.error_message.borrow_mut() = download.error_message.clone();
        imp.connections.set(download.connections);
        imp.seeders.set(download.seeders);

        // Notify property changes
        self.notify("progress");
        self.notify("status-text");
        self.notify("download-speed");
        self.notify("completed-size");
        self.notify("status");
    }

    pub fn gid(&self) -> String {
        self.imp().gid.borrow().clone()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn status(&self) -> DownloadState {
        u32_to_status(self.imp().status.get())
    }

    pub fn download_type(&self) -> DownloadType {
        u32_to_download_type(self.imp().download_type.get())
    }

    pub fn progress(&self) -> f64 {
        let total = self.imp().total_size.get();
        let completed = self.imp().completed_size.get();
        if total == 0 {
            0.0
        } else {
            completed as f64 / total as f64
        }
    }

    pub fn download_speed(&self) -> u64 {
        self.imp().download_speed.get()
    }

    pub fn upload_speed(&self) -> u64 {
        self.imp().upload_speed.get()
    }

    pub fn total_size(&self) -> u64 {
        self.imp().total_size.get()
    }

    pub fn completed_size(&self) -> u64 {
        self.imp().completed_size.get()
    }

    pub fn save_path(&self) -> String {
        self.imp().save_path.borrow().clone()
    }

    pub fn error_message(&self) -> Option<String> {
        self.imp().error_message.borrow().clone()
    }

    pub fn seeders(&self) -> u32 {
        self.imp().seeders.get()
    }

    pub fn connections(&self) -> u32 {
        self.imp().connections.get()
    }
}

fn download_type_to_u32(dt: DownloadType) -> u32 {
    match dt {
        DownloadType::Http => 0,
        DownloadType::Ftp => 1,
        DownloadType::Torrent => 2,
        DownloadType::Magnet => 3,
    }
}

fn u32_to_download_type(v: u32) -> DownloadType {
    match v {
        0 => DownloadType::Http,
        1 => DownloadType::Ftp,
        2 => DownloadType::Torrent,
        3 => DownloadType::Magnet,
        _ => DownloadType::Http,
    }
}

fn status_to_u32(s: DownloadState) -> u32 {
    match s {
        DownloadState::Active => 0,
        DownloadState::Waiting => 1,
        DownloadState::Paused => 2,
        DownloadState::Complete => 3,
        DownloadState::Error => 4,
        DownloadState::Removed => 5,
    }
}

fn u32_to_status(v: u32) -> DownloadState {
    match v {
        0 => DownloadState::Active,
        1 => DownloadState::Waiting,
        2 => DownloadState::Paused,
        3 => DownloadState::Complete,
        4 => DownloadState::Error,
        5 => DownloadState::Removed,
        _ => DownloadState::Waiting,
    }
}

fn get_status_text(status: u32, speed: u64) -> String {
    match status {
        0 => {
            if speed > 0 {
                "Downloading".to_string()
            } else {
                "Connecting".to_string()
            }
        }
        1 => "Queued".to_string(),
        2 => "Paused".to_string(),
        3 => "Complete".to_string(),
        4 => "Error".to_string(),
        5 => "Removed".to_string(),
        _ => "Unknown".to_string(),
    }
}
