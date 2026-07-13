use gtk4 as gtk;
use std::{collections::HashMap, path};

use crate::utils;

pub struct ActiveData {
    pub index: usize,
    pub element: Option<gtk::Widget>,
}

pub struct AppState {
    pub label_path_map: HashMap<path::PathBuf, gtk::Widget>,
    pub result_container: Option<gtk::Box>,
    pub render_preset: utils::RenderPreset,
    pub last_search_info: Option<dir_search_rs::LastRunInfo>,
    pub active_data: ActiveData,
}

impl AppState {
    pub fn new(render_preset: utils::RenderPreset) -> Self {
        Self {
            label_path_map: HashMap::new(),
            result_container: None,
            render_preset,
            last_search_info: None,
            active_data: ActiveData {
                index: 0,
                element: None,
            },
        }
    }
}
