use gtk4 as gtk;
use std::path;

use crate::utils;

pub struct ActiveData {
    pub active_index: usize,
    pub active_element: Option<gtk::Widget>,
    pub render_from_index: usize,
    pub rendered_items: Option<Vec<path::PathBuf>>,
}

pub struct AppState {
    pub result_container: Option<gtk::Box>,
    pub render_preset: utils::RenderPreset,
    pub last_search_info: Option<dir_search_rs::LastRunInfo>,
    pub render_data: ActiveData,
}

impl AppState {
    pub fn new(render_preset: utils::RenderPreset) -> Self {
        Self {
            result_container: None,
            render_preset,
            last_search_info: None,
            render_data: ActiveData {
                active_index: 0,
                active_element: None,
                render_from_index: 0,
                rendered_items: None,
            },
        }
    }
}
