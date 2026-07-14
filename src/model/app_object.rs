use std::path;

use gtk4 as gtk;

use gtk::{
    gio, gio::prelude::ApplicationCommandLineExtManual, glib, prelude::*, subclass::prelude::*,
};

use crate::model::desktop_entry::DesktopEntry;

pub enum EntryData {
    DesktopFile(DesktopEntry),
    Image(path::PathBuf),
}

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct AppObject {
        pub entry: RefCell<Option<EntryData>>,
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
    pub fn new(entry: EntryData) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().entry.replace(Some(entry));
        obj
    }

    pub fn name(&self) -> Option<String> {
        self.imp()
            .entry
            .borrow()
            .as_ref()
            .map(|entry| {
                if let EntryData::DesktopFile(entry) = entry {
                    Some(entry.name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn icon(&self) -> Option<String> {
        self.imp().entry.borrow().as_ref().and_then(|entry| {
            if let EntryData::DesktopFile(entry) = entry {
                entry.icon.clone()
            } else {
                None
            }
        })
    }

    pub fn launch(
        &self,
        term_exec: Option<&str>,
        cli_connection: Option<&gio::ApplicationCommandLine>,
    ) -> std::io::Result<()> {
        let imp = self.imp();
        match imp.entry.borrow().as_ref() {
            Some(EntryData::DesktopFile(entry)) => entry.launch(term_exec),
            Some(EntryData::Image(path)) => {
                if let Some(cmd) = cli_connection {
                    cmd.print_literal(&format!("{}\n", path.display()));
                    cmd.set_exit_code(glib::ExitCode::SUCCESS);
                }

                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn get_img_path(&self) -> Option<path::PathBuf> {
        let imp = self.imp();
        let entry = imp.entry.borrow();
        match &*entry {
            Some(EntryData::Image(path)) => Some(path.clone()),
            _ => None,
        }
    }
}
