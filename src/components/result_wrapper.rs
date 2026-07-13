use gtk4 as gtk;
use std::borrow::Borrow;
use std::{cell::RefCell, rc::Rc};

use crate::app_state;
use crate::constants::css_classes;
use crate::modules::search;

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> gtk::Box {
    let result_wrapper = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .css_classes([css_classes::RESULT_WRAPPER])
        .build();

    let app_state_mut_borrow = &mut the_app_state.borrow_mut();
    app_state_mut_borrow.result_container = Some(result_wrapper.clone());

    search::handle_search_and_render(app_state_mut_borrow, config.borrow(), "");

    result_wrapper
}
