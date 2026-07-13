use gtk4 as gtk;
use std::{fs, path};

pub fn create_element(result: &path::PathBuf) -> Option<gtk::Label> {
    let search_for = "Name=";

    fs::read_to_string(result)
        .map(|file_data| {
            file_data.find(search_for).map(|index| {
                let index = index + search_for.len();
                let end = file_data[index..]
                    .find('\n')
                    .map(|found_index| found_index + index)
                    .unwrap_or(file_data.len());
                file_data[index..end].to_string()
            })
        })
        .map(|name| {
            name.map(|inner| {
                gtk::Label::builder()
                    .label(inner)
                    .hexpand(true)
                    .xalign(0.0)
                    .build()
            })
        })
        .unwrap_or(None)
}
