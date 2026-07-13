use gtk4::{self as gtk, glib, prelude::EditableExt};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state;
use crate::components::result_list::{self, ListHandles};
use crate::constants::css_classes;

pub fn create_element(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    config: &Rc<dir_search_rs::ParseConfig>,
    handles: ListHandles,
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
        #[strong]
        handles,
        move |entry_widget| {
            let search_text = entry_widget.text().to_string();
            let search_text = search_text.trim();

            let state = &mut the_app_state.borrow_mut();
            result_list::populate(&handles, state, config.borrow(), search_text);
        }
    ));

    input_entry
}
