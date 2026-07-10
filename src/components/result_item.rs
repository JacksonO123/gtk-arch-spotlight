use gtk4 as gtk;
use std::path;

use crate::constants::css_classes;

pub fn create_element(result: &path::PathBuf) -> gtk::Label {
    let label = gtk::Label::builder()
        .label(result.to_str().unwrap())
        .hexpand(true)
        .halign(gtk::Align::Start)
        .css_classes([css_classes::RESULT_ITEM])
        .build();

    label
}
