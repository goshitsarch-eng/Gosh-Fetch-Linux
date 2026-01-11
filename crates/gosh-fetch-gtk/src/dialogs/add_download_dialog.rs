//! Add Download Dialog - dialog for adding new downloads

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;

use crate::window::GoshFetchWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AddDownloadDialog {
        pub window: RefCell<Option<GoshFetchWindow>>,
        pub url_entry: RefCell<Option<gtk::Entry>>,
        pub magnet_text: RefCell<Option<gtk::TextView>>,
        pub torrent_path: RefCell<Option<String>>,
        pub torrent_label: RefCell<Option<gtk::Label>>,
        pub stack: RefCell<Option<adw::ViewStack>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddDownloadDialog {
        const NAME: &'static str = "AddDownloadDialog";
        type Type = super::AddDownloadDialog;
        type ParentType = adw::Dialog;
    }

    impl ObjectImpl for AddDownloadDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for AddDownloadDialog {}
    impl AdwDialogImpl for AddDownloadDialog {}
}

glib::wrapper! {
    pub struct AddDownloadDialog(ObjectSubclass<imp::AddDownloadDialog>)
        @extends adw::Dialog, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AddDownloadDialog {
    pub fn new(window: &GoshFetchWindow) -> Self {
        let dialog: Self = glib::Object::new();
        *dialog.imp().window.borrow_mut() = Some(window.clone());
        dialog
    }

    fn setup_ui(&self) {
        self.set_title("Add Download");
        self.set_content_width(500);
        self.set_content_height(300);

        // Main content box
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();
        header.set_show_start_title_buttons(false);
        header.set_show_end_title_buttons(false);

        let cancel_btn = gtk::Button::with_label("Cancel");
        cancel_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.close();
            }
        ));
        header.pack_start(&cancel_btn);

        let add_btn = gtk::Button::with_label("Add");
        add_btn.add_css_class("suggested-action");
        add_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.add_download();
            }
        ));
        header.pack_end(&add_btn);

        content.append(&header);

        // View stack switcher
        let stack = adw::ViewStack::new();
        *self.imp().stack.borrow_mut() = Some(stack.clone());

        let switcher = adw::ViewSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
        switcher.set_margin_start(16);
        switcher.set_margin_end(16);
        switcher.set_margin_top(8);
        switcher.set_margin_bottom(8);
        content.append(&switcher);

        // URL tab
        let url_page = self.create_url_page();
        stack.add_titled_with_icon(&url_page, Some("url"), "URL", "web-browser-symbolic");

        // Magnet tab
        let magnet_page = self.create_magnet_page();
        stack.add_titled_with_icon(
            &magnet_page,
            Some("magnet"),
            "Magnet",
            "network-transmit-receive-symbolic",
        );

        // Torrent file tab
        let torrent_page = self.create_torrent_page();
        stack.add_titled_with_icon(
            &torrent_page,
            Some("torrent"),
            "Torrent File",
            "document-open-symbolic",
        );

        content.append(&stack);

        self.set_child(Some(&content));
    }

    fn create_url_page(&self) -> gtk::Box {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_start(16);
        page.set_margin_end(16);
        page.set_margin_top(16);
        page.set_margin_bottom(16);

        let label = gtk::Label::new(Some("Enter URL to download"));
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        page.append(&label);

        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some("https://example.com/file.zip"));
        entry.set_hexpand(true);
        *self.imp().url_entry.borrow_mut() = Some(entry.clone());
        page.append(&entry);

        let help = gtk::Label::new(Some("Supports HTTP, HTTPS, FTP, and magnet links"));
        help.set_halign(gtk::Align::Start);
        help.add_css_class("dim-label");
        help.add_css_class("caption");
        page.append(&help);

        page
    }

    fn create_magnet_page(&self) -> gtk::Box {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_start(16);
        page.set_margin_end(16);
        page.set_margin_top(16);
        page.set_margin_bottom(16);

        let label = gtk::Label::new(Some("Enter magnet link"));
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        page.append(&label);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_min_content_height(100);

        let text_view = gtk::TextView::new();
        text_view.set_wrap_mode(gtk::WrapMode::WordChar);
        text_view.set_accepts_tab(false);
        *self.imp().magnet_text.borrow_mut() = Some(text_view.clone());

        scrolled.set_child(Some(&text_view));
        page.append(&scrolled);

        let help = gtk::Label::new(Some("Paste your magnet:?xt=urn:btih:... link here"));
        help.set_halign(gtk::Align::Start);
        help.add_css_class("dim-label");
        help.add_css_class("caption");
        page.append(&help);

        page
    }

    fn create_torrent_page(&self) -> gtk::Box {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_start(16);
        page.set_margin_end(16);
        page.set_margin_top(16);
        page.set_margin_bottom(16);

        let label = gtk::Label::new(Some("Select a .torrent file"));
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        page.append(&label);

        let file_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let file_label = gtk::Label::new(Some("No file selected"));
        file_label.set_hexpand(true);
        file_label.set_halign(gtk::Align::Start);
        file_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        *self.imp().torrent_label.borrow_mut() = Some(file_label.clone());
        file_box.append(&file_label);

        let browse_btn = gtk::Button::with_label("Browse");
        browse_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.browse_torrent_file();
            }
        ));
        file_box.append(&browse_btn);

        page.append(&file_box);

        page
    }

    fn browse_torrent_file(&self) {
        let dialog = gtk::FileDialog::new();
        dialog.set_title("Select Torrent File");

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.torrent");
        filter.set_name(Some("Torrent Files"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        let self_weak = self.downgrade();
        dialog.open(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            move |result| {
                if let Some(dialog) = self_weak.upgrade() {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            *dialog.imp().torrent_path.borrow_mut() = Some(path_str.clone());
                            if let Some(label) = dialog.imp().torrent_label.borrow().as_ref() {
                                label.set_text(&path_str);
                            }
                        }
                    }
                }
            },
        );
    }

    fn add_download(&self) {
        let imp = self.imp();

        // Get current tab
        let stack = imp.stack.borrow();
        let current_page = stack.as_ref().and_then(|s| s.visible_child_name());

        let window = imp.window.borrow();
        let window = match window.as_ref() {
            Some(w) => w,
            None => return,
        };

        match current_page.as_ref().map(|s| s.as_str()) {
            Some("url") => {
                if let Some(entry) = imp.url_entry.borrow().as_ref() {
                    let url = entry.text().to_string();
                    if !url.is_empty() {
                        // Check if it's a magnet link
                        if url.starts_with("magnet:") {
                            window.add_magnet(&url);
                        } else {
                            window.add_url(&url);
                        }
                        self.close();
                    }
                }
            }

            Some("magnet") => {
                if let Some(text_view) = imp.magnet_text.borrow().as_ref() {
                    let buffer = text_view.buffer();
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    let uri = buffer.text(&start, &end, false).to_string();
                    if !uri.is_empty() && uri.starts_with("magnet:") {
                        window.add_magnet(&uri);
                        self.close();
                    }
                }
            }

            Some("torrent") => {
                if let Some(path) = imp.torrent_path.borrow().as_ref() {
                    // Read torrent file
                    if let Ok(data) = std::fs::read(path) {
                        window.add_torrent(&data);
                        self.close();
                    }
                }
            }

            _ => {}
        }
    }
}
