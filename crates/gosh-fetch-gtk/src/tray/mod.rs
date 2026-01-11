//! System tray module

use ksni::{menu::StandardItem, MenuItem, Tray, TrayService};
use std::sync::{Arc, Mutex};

use gosh_fetch_core::{format_speed, GlobalStats};

/// Tray icon implementation
pub struct GoshFetchTray {
    stats: Arc<Mutex<GlobalStats>>,
    show_window: Box<dyn Fn() + Send + Sync>,
    hide_window: Box<dyn Fn() + Send + Sync>,
    pause_all: Box<dyn Fn() + Send + Sync>,
    resume_all: Box<dyn Fn() + Send + Sync>,
    quit: Box<dyn Fn() + Send + Sync>,
}

impl GoshFetchTray {
    pub fn new<F1, F2, F3, F4, F5>(
        show_window: F1,
        hide_window: F2,
        pause_all: F3,
        resume_all: F4,
        quit: F5,
    ) -> Self
    where
        F1: Fn() + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
        F3: Fn() + Send + Sync + 'static,
        F4: Fn() + Send + Sync + 'static,
        F5: Fn() + Send + Sync + 'static,
    {
        Self {
            stats: Arc::new(Mutex::new(GlobalStats::default())),
            show_window: Box::new(show_window),
            hide_window: Box::new(hide_window),
            pause_all: Box::new(pause_all),
            resume_all: Box::new(resume_all),
            quit: Box::new(quit),
        }
    }

    pub fn update_stats(&self, stats: GlobalStats) {
        if let Ok(mut s) = self.stats.lock() {
            *s = stats;
        }
    }
}

impl Tray for GoshFetchTray {
    fn icon_name(&self) -> String {
        "folder-download".to_string()
    }

    fn title(&self) -> String {
        "Gosh-Fetch".to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let stats = self.stats.lock().map(|s| s.clone()).unwrap_or_default();
        let dl = format_speed(stats.download_speed);
        let ul = format_speed(stats.upload_speed);

        ksni::ToolTip {
            icon_name: "folder-download".to_string(),
            title: "Gosh-Fetch".to_string(),
            description: format!("↓ {} ↑ {}\n{} active", dl, ul, stats.num_active),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Show Window".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    (tray.show_window)();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Hide Window".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    (tray.hide_window)();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Pause All".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    (tray.pause_all)();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Resume All".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    (tray.resume_all)();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    (tray.quit)();
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        (self.show_window)();
    }
}

/// Start the tray service
pub fn start_tray_service<F1, F2, F3, F4, F5>(
    show_window: F1,
    hide_window: F2,
    pause_all: F3,
    resume_all: F4,
    quit: F5,
) -> Option<TrayService<GoshFetchTray>>
where
    F1: Fn() + Send + Sync + 'static,
    F2: Fn() + Send + Sync + 'static,
    F3: Fn() + Send + Sync + 'static,
    F4: Fn() + Send + Sync + 'static,
    F5: Fn() + Send + Sync + 'static,
{
    let tray = GoshFetchTray::new(show_window, hide_window, pause_all, resume_all, quit);

    // TrayService::new returns the service directly
    Some(TrayService::new(tray))
}
