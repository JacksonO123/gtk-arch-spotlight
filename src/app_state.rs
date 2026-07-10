use gtk4 as gtk;
use std::{collections::HashMap, path};

pub struct AppState {
    pub label_path_map: HashMap<path::PathBuf, gtk::Revealer>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            label_path_map: HashMap::new(),
        }
    }
}
