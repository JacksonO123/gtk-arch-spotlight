use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk4 as gtk;
use gtk4_layer_shell::LayerShell;
use std::{cell::RefCell, rc::Rc};

mod constants;
mod utils;

use constants::css_classes;

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("com.jackson.spotlight")
        .build();

    app.connect_startup(|_| utils::load_css());
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Spotlight")
            .css_classes([css_classes::OVERLAY_ROOT])
            .build();

        let parse_config = Rc::new(RefCell::new(dir_search_rs::ParseConfig {
            search_dir: "/home/jotto/code/window-utils/spotlight/test-data".to_string(),
            search_str: "{search}".to_string(),
            search_contents: dir_search_rs::SearchContents::FileName,
            parallel_preference: None,
        }));

        // window.init_layer_shell();
        // window.set_layer(gtk4_layer_shell::Layer::Overlay);

        let fill = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .css_classes([css_classes::OVERLAY_FILL])
            .build();

        let input_entry = gtk::Entry::builder()
            .hexpand(true)
            .css_classes([css_classes::SEARCH_INPUT])
            .build();

        let window_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .vexpand(true)
            .css_classes([css_classes::WINDOW_CONTENTS])
            .build();
        window_content.append(&input_entry);

        let result_wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(constants::CONTENT_GAP)
            .valign(gtk::Align::Center)
            .css_classes([css_classes::RESULT_WRAPPER])
            .build();
        window_content.append(&result_wrapper);

        input_entry.connect_changed(glib::clone!(
            #[strong]
            parse_config,
            #[weak]
            result_wrapper,
            move |entry_widget| {
                let search_text = entry_widget.text().to_string();
                let search_text = search_text.trim();
                match dir_search_rs::search_with_config(&parse_config.borrow(), search_text) {
                    Ok(res) => {
                        render_results(&result_wrapper, &res);
                    }
                    Err(err) => crate::error_log!(err),
                }
            }
        ));

        match dir_search_rs::search_with_config(&parse_config.borrow(), "") {
            Ok(res) => {
                render_results(&result_wrapper, &res);
            }
            Err(err) => crate::error_log!(err),
        }

        fill.append(&window_content);

        window.set_child(Some(&fill));

        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);

        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        click.connect_pressed(glib::clone!(
            #[weak]
            window,
            #[weak]
            window_content,
            move |_, _, x, y| {
                if let Some(bounds) = window_content.compute_bounds(&window)
                    && !bounds.contains_point(&gtk::graphene::Point::new(x as f32, y as f32))
                {
                    handle_close_window(&window);
                }
            }
        ));
        window.add_controller(click);

        let key = gtk::EventControllerKey::new();
        key.connect_key_pressed(glib::clone!(
            #[weak]
            window,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    handle_close_window(&window);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        window.add_controller(key);

        window.present();
    });

    app.run()
}

fn handle_close_window(window: &gtk::ApplicationWindow) {
    // window.hide();
    window.close();
}

fn render_results(result_container: &gtk::Box, results: &Vec<std::path::PathBuf>) {
    while let Some(child) = result_container.first_child() {
        result_container.remove(&child);
    }

    for result in results {
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
    }
}
