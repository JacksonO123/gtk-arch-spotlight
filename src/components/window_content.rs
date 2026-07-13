use gtk4 as gtk;
use gtk4::prelude::BoxExt;
use std::{cell::RefCell, rc::Rc};

use crate::app_state;
use crate::components::input_entry;
use crate::components::result_list::{self, ListHandles};
use crate::constants::css_classes;

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> (gtk::Box, ListHandles) {
    let window_content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Start)
        .vexpand(true)
        .css_classes([css_classes::WINDOW_CONTENTS])
        .build();

    let (result_scroller, handles) = result_list::create_element(the_app_state, config);
    let input_entry_element = input_entry::create_element(the_app_state, config, handles.clone());

    window_content.append(&input_entry_element);
    window_content.append(&result_scroller);

    (window_content, handles)
}
