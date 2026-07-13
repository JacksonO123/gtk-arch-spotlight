use gtk4 as gtk;

use gtk::glib;
use gtk::subclass::prelude::*;

use crate::model::desktop_entry::DesktopEntry;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct AppObject {
        pub entry: RefCell<Option<DesktopEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppObject {
        const NAME: &'static str = "SpotlightAppObject";
        type Type = super::AppObject;
    }

    impl ObjectImpl for AppObject {}
}

glib::wrapper! {
    pub struct AppObject(ObjectSubclass<imp::AppObject>);
}

impl AppObject {
    pub fn new(entry: DesktopEntry) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().entry.replace(Some(entry));
        obj
    }

    pub fn name(&self) -> String {
        self.imp()
            .entry
            .borrow()
            .as_ref()
            .map(|entry| entry.name.clone())
            .unwrap_or_default()
    }

    pub fn icon(&self) -> Option<String> {
        self.imp()
            .entry
            .borrow()
            .as_ref()
            .and_then(|entry| entry.icon.clone())
    }

    pub fn launch(&self, term_exec: Option<&str>) -> std::io::Result<()> {
        match self.imp().entry.borrow().as_ref() {
            Some(entry) => entry.launch(term_exec),
            None => Ok(()),
        }
    }
}
