use crate::utils;

/// Logical application state.
///
/// The view (list model, selection, `ListView`) is owned by the widget tree
/// and threaded around via [`crate::components::result_list::ListHandles`], so
/// this holds only the non-widget state the search flow needs.
pub struct AppState {
    pub render_preset: utils::RenderPreset,
    pub last_search_info: Option<dir_search_rs::LastRunInfo>,
}

impl AppState {
    pub fn new(render_preset: utils::RenderPreset) -> Self {
        Self {
            render_preset,
            last_search_info: None,
        }
    }
}
