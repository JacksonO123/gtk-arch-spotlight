use gtk4 as gtk;
use gtk4::prelude::BoxExt;
use std::{cell::RefCell, rc::Rc};

use crate::app_state;
use crate::components::{input_entry, result_wrapper};
use crate::constants::css_classes;

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> gtk::Box {
    let window_content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Start)
        .vexpand(true)
        .css_classes([css_classes::WINDOW_CONTENTS])
        .build();

    let result_wrapper_element = result_wrapper::create_element(the_app_state, config);
    let input_entry_element = input_entry::create_element(the_app_state, config);

    window_content.append(&input_entry_element);
    window_content.append(&result_wrapper_element);

    window_content
}
