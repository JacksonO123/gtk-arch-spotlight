use gtk4::{
    self as gtk,
    glib::{self, object::Cast},
    prelude::{BoxExt, WidgetExt},
};
use std::{collections::HashSet, fs, path};

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

    let mut current_results_set: HashSet<path::PathBuf> =
        results.iter().map(|item| item.path()).collect();

    let (mut filtered_current_rendered, new_result_paths): (Vec<_>, Vec<_>) = {
        let mut current_child = skip_closed_revealer(result_container.first_child());
        let current_rendered_items = the_app_state
            .render_data
            .rendered_items
            .take()
            .unwrap_or_default();
        let filtered_items_set: Vec<_> = current_rendered_items
            .into_iter()
            .filter(|item| {
                let contains = current_results_set.contains(item);

                let next_child = current_child
                    .clone()
                    .and_then(|child| skip_closed_revealer(child.next_sibling()));

                if !contains && let Some(child) = &current_child {
                    if flags::ANIMATION_ENABLED {
                        unsafe {
                            child
                                .clone()
                                .unsafe_cast::<gtk::Revealer>()
                                .set_reveal_child(false);
                        }
                    } else {
                        result_container.remove(child);
                    }
                }

                current_child = next_child;

                contains
            })
            .collect();

        current_results_set.retain(|item| !filtered_items_set.contains(item));
        let filtered_items_vec: Vec<_> = filtered_items_set.into_iter().collect();
        let new_paths: Vec<_> = current_results_set.into_iter().collect();

        (filtered_items_vec, new_paths)
    };

    let needs = constants::MAX_RESULTS
        - std::cmp::min(filtered_current_rendered.len(), constants::MAX_RESULTS);

    let mut items_to_add: Vec<_> =
        new_result_paths[0..std::cmp::min(new_result_paths.len(), needs)].to_vec();

    for result in items_to_add.iter() {
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

        widget.add_css_class(css_classes::RESULT_ITEM);

        let item_widget = if flags::ANIMATION_ENABLED {
            let wrapper_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();
            wrapper_box.append(&widget);

            let widget = gtk::Revealer::builder()
                .child(&wrapper_box)
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
    }

    filtered_current_rendered.append(&mut items_to_add);
    the_app_state.render_data.rendered_items = Some(filtered_current_rendered);

    if results.is_empty() {
        result_container.add_css_class(css_classes::EMPTY);
    } else {
        result_container.remove_css_class(css_classes::EMPTY);
    }

    skip_closed_revealer(result_container.first_child()).inspect(move |element| {
        toggle_active_element(element.clone(), true);
        the_app_state.render_data.active_element = Some(element.clone());
    })
}

fn skip_closed_revealer(mut widget: Option<gtk::Widget>) -> Option<gtk::Widget> {
    if !flags::ANIMATION_ENABLED {
        return widget;
    }

    while let Some(element) = &widget {
        unsafe {
            if element.unsafe_cast_ref::<gtk::Revealer>().reveals_child() {
                return Some(element.clone());
            }

            widget = element.next_sibling();
        }
    }

    None
}

pub fn toggle_active_element(widget: gtk::Widget, value: bool) {
    let target_element = if flags::ANIMATION_ENABLED {
        widget
            .first_child()
            .and_then(|wrapper| wrapper.first_child())
    } else {
        Some(widget)
    };

    if let Some(target) = target_element {
        if value {
            target.add_css_class(css_classes::ACTIVE_RESULT);
        } else {
            target.remove_css_class(css_classes::ACTIVE_RESULT);
        }
    }
}
