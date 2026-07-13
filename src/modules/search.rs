use std::collections::HashMap;
use std::fs;

use crate::error_log;
use crate::model::{AppObject, desktop_entry::DesktopEntry};
use crate::utils::RenderPreset;

/// Run a search and produce the model items to display.
///
/// Owns the interaction with `dir_search_rs` (including the incremental
/// "reuse last run" optimisation, threaded through `last_search_info`) and
/// turns the raw `DirEntry` hits into deduplicated, sorted [`AppObject`]s ready
/// to be spliced into the list store.
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
        // Image rendering is not implemented yet; produce nothing rather than
        // panicking so the rest of the UI stays functional.
        RenderPreset::Images => Vec::new(),
    };

    *last_search_info = Some(dir_search_rs::LastRunInfo {
        last_run_search_str_len: search_text.len(),
        last_run_results: raw,
    });

    items
}

/// Parse each result once, collapse duplicates by (case-insensitive) name, and
/// sort alphabetically for a stable ordering.
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
