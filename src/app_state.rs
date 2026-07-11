use gtk4 as gtk;
use std::{collections::HashMap, path};

use crate::utils;

pub struct AppState {
    pub label_path_map: HashMap<path::PathBuf, gtk::Revealer>,
    pub result_container: Option<gtk::Box>,
    pub render_preset: utils::RenderPreset,
    pub last_search_info: Option<dir_search_rs::LastRunInfo>,
}

impl AppState {
    pub fn new(render_preset: utils::RenderPreset) -> Self {
        Self {
            label_path_map: HashMap::new(),
            result_container: None,
            render_preset,
            last_search_info: None,
        }
    }
}
