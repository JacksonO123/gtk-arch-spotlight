use gtk4 as gtk;
use std::fs;

use crate::constants::css_classes;

pub fn create_element(result: &fs::DirEntry) -> Option<gtk::Label> {
    let search_for = "Name=";

    fs::read_to_string(result.path())
        .map(|file_data| {
            file_data.find(search_for).map(|index| {
                let index = index + search_for.len();
                let end = file_data[index..]
                    .find("\n")
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
                    .halign(gtk::Align::Start)
                    .css_classes([css_classes::RESULT_ITEM])
                    .build()
            })
        })
        .unwrap_or(None)
}
