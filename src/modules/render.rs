use gtk4::{
    self as gtk,
    glib::{self, object::Cast},
    prelude::{BoxExt, WidgetExt},
};
use std::{collections::HashSet, fs};

use crate::{
    app_state,
    components::result_item,
    constants::{self, css_classes},
    error_log, flags, utils,
};

pub fn render_results(
    the_app_state: &mut app_state::AppState,
    results: &[fs::DirEntry],
) -> Option<gtk::Widget> {
    let result_container = &the_app_state.result_container;
    if result_container.is_none() {
        error_log!("Result container is none");
        return None;
    }
    let result_container = result_container.as_ref().unwrap();

    let current_results_set: HashSet<_> = results.iter().map(|item| item.path()).collect();

    the_app_state.label_path_map.retain(|key, value| {
        let res = current_results_set.contains(key);

        if !res {
            if flags::ANIMATION_ENABLED {
                unsafe {
                    value
                        .clone()
                        .unsafe_cast::<gtk::Revealer>()
                        .set_reveal_child(false);
                }
            } else {
                result_container.remove(value);
            }
        }

        res
    });
    let shown_len = the_app_state.label_path_map.len();
    let needs = constants::MAX_RESULTS - std::cmp::min(shown_len, constants::MAX_RESULTS);

    let rendered_items: Vec<_> = results[0..std::cmp::min(results.len(), needs)]
        .iter()
        .collect();
    for result in rendered_items.iter() {
        if the_app_state.label_path_map.contains_key(&result.path()) {
            continue;
        }

        let widget: Option<gtk::Widget> = match the_app_state.render_preset {
            utils::RenderPreset::DesktopFile => {
                result_item::create_element(result).map(|widget| widget.upcast())
            }
            utils::RenderPreset::Images => {
                unreachable!()
            }
        };

        if widget.is_none() {
            continue;
        }
        let widget = widget.unwrap();

        let item_widget = if flags::ANIMATION_ENABLED {
            let widget = gtk::Revealer::builder()
                .child(&widget)
                .transition_type(gtk::RevealerTransitionType::SlideUp)
                .transition_duration(constants::ANIMATION_DURATION_MS)
                .hexpand(true)
                .reveal_child(false)
                .build();

            widget.connect_child_revealed_notify(glib::clone!(
                #[weak]
                result_container,
                move |revealer| {
                    if !revealer.is_child_revealed() {
                        result_container.remove(revealer);
                    }
                },
            ));

            widget.upcast()
        } else {
            widget
        };

        result_container.append(&item_widget);

        if flags::ANIMATION_ENABLED {
            unsafe {
                item_widget
                    .clone()
                    .unsafe_cast::<gtk::Revealer>()
                    .set_reveal_child(true);
            }
        }

        the_app_state
            .label_path_map
            .insert(result.path(), item_widget);
    }

    if results.is_empty() {
        result_container.add_css_class(css_classes::EMPTY);
    } else {
        result_container.remove_css_class(css_classes::EMPTY);
    }

    result_container.first_child().map(move |element| {
        element.add_css_class(css_classes::ACTIVE_RESULT);
        the_app_state.active_data.element = Some(element.clone());
        element
    })
}
