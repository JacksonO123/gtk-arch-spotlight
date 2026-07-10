use gtk4::{self as gtk, glib, prelude::BoxExt};
use std::{cell::RefCell, collections::HashSet, path, rc::Rc};

use crate::{
    app_state,
    constants::{self, css_classes},
};

pub fn render_results(
    the_app_state: &Rc<RefCell<app_state::AppState>>,
    result_container: &gtk::Box,
    results: Vec<path::PathBuf>,
) {
    let mut current_results_set: HashSet<path::PathBuf> =
        results.iter().map(|item| item.to_owned()).collect();

    for (key, value) in the_app_state.borrow().label_path_map.iter() {
        if !current_results_set.contains(key) {
            value.set_reveal_child(false);
        }

        current_results_set.remove(key);
    }

    for result in current_results_set {
        let label = gtk::Label::builder()
            .label(result.to_str().unwrap())
            .hexpand(true)
            .halign(gtk::Align::Start)
            .css_classes([css_classes::RESULT_ITEM])
            .build();

        let label_revealer = gtk::Revealer::builder()
            .child(&label)
            .transition_type(gtk::RevealerTransitionType::SlideUp)
            .transition_duration(constants::ANIMATION_DURATION_MS)
            .hexpand(true)
            .build();

        let pre_move_result_clone = result.to_owned();

        label_revealer.connect_child_revealed_notify(glib::clone!(
            #[weak]
            result_container,
            #[strong]
            the_app_state,
            move |revealer| {
                if !revealer.is_child_revealed() {
                    let the_app_state: &mut app_state::AppState = &mut the_app_state.borrow_mut();
                    the_app_state.label_path_map.remove(&result);
                    result_container.remove(revealer);
                }
            },
        ));

        result_container.append(&label_revealer);
        label_revealer.set_reveal_child(true);

        let mut temp = the_app_state.borrow_mut();
        temp.label_path_map
            .insert(pre_move_result_clone, label_revealer);
    }
}
