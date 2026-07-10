use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk4 as gtk;
use gtk4_layer_shell::LayerShell;
use std::{cell::RefCell, rc::Rc};

mod app_state;
mod components;
mod constants;
mod render;
mod utils;

use components::fill;
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

        let parse_config = Rc::new(dir_search_rs::ParseConfig {
            search_dir: "/home/jotto/code/window-utils/spotlight/test-data".to_string(),
            search_str: "{search}".to_string(),
            search_contents: dir_search_rs::SearchContents::FileName,
            parallel_preference: None,
        });

        // window.init_layer_shell();
        // window.set_layer(gtk4_layer_shell::Layer::Overlay);

        let the_app_state = Rc::new(RefCell::new(app_state::AppState::new()));

        let (fill_element, window_content_element) =
            fill::create_element(&the_app_state, &parse_config);

        window.set_child(Some(&fill_element));

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
            window_content_element,
            move |_, _, x, y| {
                if let Some(bounds) = window_content_element.compute_bounds(&window)
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
