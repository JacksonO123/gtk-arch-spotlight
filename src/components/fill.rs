use gtk4 as gtk;
use gtk4::prelude::BoxExt;
use std::{cell::RefCell, rc::Rc};

use crate::app_state;
use crate::components::window_content;
use crate::constants::css_classes;

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> (gtk::Box, gtk::Box) {
    let fill = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .css_classes([css_classes::OVERLAY_FILL])
        .build();

    let window_content_element = window_content::create_element(the_app_state, config);

    fill.append(&window_content_element);

    (fill, window_content_element)
}
