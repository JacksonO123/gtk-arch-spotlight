use std::collections::HashMap;
use std::fs;

use crate::error_log;
use crate::model::{AppObject, desktop_entry::DesktopEntry};
use crate::utils::RenderPreset;

pub fn run_search(
    preset: RenderPreset,
    config: &dir_search_rs::ParseConfig,
    last_search_info: &mut Option<dir_search_rs::LastRunInfo>,
    search_text: &str,
) -> Vec<AppObject> {
    let previous = last_search_info.take();

    let raw = match dir_search_rs::search_with_config(config, search_text, previous) {
        Ok(raw) => raw,
        Err(err) => {
            error_log!(err);
            return Vec::new();
        }
    };

    let items = match preset {
        RenderPreset::DesktopFile => build_desktop_items(&raw),
        RenderPreset::Images => Vec::new(),
    };

    *last_search_info = Some(dir_search_rs::LastRunInfo {
        last_run_search_str_len: search_text.len(),
        last_run_results: raw,
    });

    items
}

fn build_desktop_items(raw: &[fs::DirEntry]) -> Vec<AppObject> {
    let mut by_name: HashMap<String, DesktopEntry> = HashMap::new();

    for entry in raw {
        if let Some(desktop) = DesktopEntry::from_path(&entry.path()) {
            by_name
                .entry(desktop.name.to_ascii_lowercase())
                .or_insert(desktop);
        }
    }

    let mut entries: Vec<DesktopEntry> = by_name.into_values().collect();
    entries.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });

    entries.into_iter().map(AppObject::new).collect()
}
