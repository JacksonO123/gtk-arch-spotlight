use gtk4 as gtk;
use std::{cell::RefCell, rc::Rc};

use crate::constants::css_classes;
use crate::{app_state, error_log, render};

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> gtk::Box {
    let result_wrapper = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .css_classes([css_classes::RESULT_WRAPPER])
        .build();

    let mut app_state_mut_borrow = the_app_state.borrow_mut();
    app_state_mut_borrow.result_container = Some(result_wrapper.clone());

    match dir_search_rs::search_with_config(config, "", None) {
        Ok(res) => {
            render::render_results(&mut app_state_mut_borrow, &res);
        }
        Err(err) => error_log!(err),
    }

    result_wrapper
}
