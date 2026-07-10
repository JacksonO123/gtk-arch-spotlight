use gtk4::{self as gtk, glib, prelude::BoxExt};
use std::{collections::HashSet, path};

use crate::{
    app_state,
    constants::{self, css_classes},
    flags,
};

pub fn render_results(
    the_app_state: &mut app_state::AppState,
    result_container: &gtk::Box,
    results: Vec<path::PathBuf>,
) {
    let current_results_set: HashSet<_> = results.iter().collect();

    the_app_state.label_path_map.retain(|key, value| {
        let res = current_results_set.contains(key);

        if !res {
            value.set_reveal_child(false);
        }

        res
    });

    for result in results {
        if the_app_state.label_path_map.contains_key(&result) {
            continue;
        }

        let label = gtk::Label::builder()
            .label(result.to_str().unwrap())
            .hexpand(true)
            .halign(gtk::Align::Start)
            .css_classes([css_classes::RESULT_ITEM])
            .build();

        let label_revealer = gtk::Revealer::builder()
            .child(&label)
            .transition_type(gtk::RevealerTransitionType::SlideUp)
            .transition_duration(if flags::ANIMATION_ENABLED {
                constants::ANIMATION_DURATION_MS
            } else {
                0
            })
            .hexpand(true)
            .build();

        let pre_move_result_clone = result.to_owned();

        label_revealer.connect_child_revealed_notify(glib::clone!(
            #[weak]
            result_container,
            move |revealer| {
                if !revealer.is_child_revealed() {
                    result_container.remove(revealer);
                }
            },
        ));

        result_container.append(&label_revealer);
        label_revealer.set_reveal_child(true);

        the_app_state
            .label_path_map
            .insert(pre_move_result_clone, label_revealer);
    }
}
