use gtk4 as gtk;

use std::collections::HashMap;
use std::fs;

use crate::app_state;
use crate::error_log;
use crate::modules::render;

pub fn handle_search_and_render(
    the_app_state: &mut app_state::AppState,
    config: &dir_search_rs::ParseConfig,
    search_text: &str,
) -> Option<gtk::Widget> {
    let res = {
        let last_search_info = the_app_state.last_search_info.take();
        dir_search_rs::search_with_config(config, search_text, last_search_info)
    };

    match res {
        Ok(res) => {
            let res = to_merged_results(res);

            let first_element = render::render_results(the_app_state, &res);

            the_app_state.last_search_info = Some(dir_search_rs::LastRunInfo {
                last_run_search_str_len: search_text.len(),
                last_run_results: res,
            });

            first_element
        }
        Err(err) => {
            error_log!(err);
            None
        }
    }
}

fn to_merged_results(results: Vec<fs::DirEntry>) -> Vec<fs::DirEntry> {
    results
        .into_iter()
        .filter_map(move |item| {
            get_desktop_property("name", &item)
                .map(|name_value| (name_value.to_ascii_lowercase(), item))
        })
        .collect::<HashMap<String, fs::DirEntry>>()
        .into_values()
        .collect::<Vec<fs::DirEntry>>()
}

fn get_desktop_property(property: &str, entry: &fs::DirEntry) -> Option<String> {
    fs::read_to_string(entry.path())
        .map(|file_data| {
            file_data
                .to_ascii_lowercase()
                .find(property)
                .map(|mut location| {
                    // account for =
                    location += property.len() + 1;
                    let end = file_data[location..]
                        .find('\n')
                        .map(|end_location| end_location + location)
                        .unwrap_or(file_data.len());
                    file_data[location..end].to_string()
                })
        })
        .unwrap_or(None)
}
