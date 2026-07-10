use gtk4::{self as gtk, glib, prelude::EditableExt};
use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::app_state;
use crate::constants::css_classes;
use crate::{error_log, render};

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
) -> gtk::Entry {
    let input_entry = gtk::Entry::builder()
        .hexpand(true)
        .css_classes([css_classes::SEARCH_INPUT])
        .build();

    input_entry.connect_changed(glib::clone!(
        #[strong]
        config,
        #[strong]
        the_app_state,
        move |entry_widget| {
            let search_text = entry_widget.text().to_string();
            let search_text = search_text.trim();
            match dir_search_rs::search_with_config(config.borrow(), search_text) {
                Ok(res) => {
                    render::render_results(&mut the_app_state.borrow_mut(), res);
                }
                Err(err) => error_log!(err),
            }
        }
    ));

    input_entry
}
