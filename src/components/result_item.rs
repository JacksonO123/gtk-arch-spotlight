use gtk4 as gtk;
use std::fs;

use crate::constants::css_classes;

pub fn create_element(result: &fs::DirEntry) -> gtk::Label {
    gtk::Label::builder()
        .label(result.file_name().to_str().unwrap())
        .hexpand(true)
        .halign(gtk::Align::Start)
        .css_classes([css_classes::RESULT_ITEM])
        .build()
}
