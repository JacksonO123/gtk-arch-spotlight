use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use gtk4 as gtk;
use gtk4_layer_shell::LayerShell;
use std::{cell::RefCell, rc::Rc};

mod app_state;
mod components;
mod constants;
mod flags;
mod render;
mod utils;

use components::fill;
use constants::css_classes;

fn main() -> glib::ExitCode {
    let args = std::env::args();
    if args.len() < 2 {
        panic!("Expected render preset");
    }

    let app = gtk::Application::builder()
        .application_id("com.jackson.spotlight")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|_| utils::load_css());

    app.connect_command_line(move |app, cmd_line| {
        let args: Vec<_> = cmd_line.arguments();

        let mut config = utils::AppConfig {
            close: false,
            render_preset: None,
        };

        for i in 1..args.len() {
            let arg = &args[i];

            match arg.to_str().unwrap() {
                "--close" => {
                    config.close = true;
                }
                "--render-desktop-files" => {
                    if config.render_preset.is_some() {
                        panic!("Duplicate render preset options found")
                    }
                    config.render_preset = Some(utils::RenderPreset::DesktopFile);
                }
                "--render-images" => {
                    if config.render_preset.is_some() {
                        panic!("Duplicate render preset options found")
                    }
                    config.render_preset = Some(utils::RenderPreset::Image);
                }
                _ => panic!("Unrecognized arg: {}", arg.to_str().unwrap()),
            }
        }

        let the_app_state = if let Some(render_preset) = config.render_preset {
            Rc::new(RefCell::new(app_state::AppState::new(render_preset)))
        } else {
            panic!("Expected render preset")
        };

        let window = match app.windows().first() {
            Some(win) => win.clone().downcast::<gtk::ApplicationWindow>().unwrap(),
            None => init_window(app, &the_app_state),
        };

        window.present();

        if config.close {
            window.set_visible(false);
        }

        glib::ExitCode::SUCCESS
    });

    app.run()
}

fn init_window(
    app: &gtk::Application,
    the_app_state: &Rc<RefCell<app_state::AppState>>,
) -> gtk::ApplicationWindow {
    _ = app.hold();

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

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);

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

    window
}

#[cfg(debug_assertions)]
fn handle_close_window(window: &gtk::ApplicationWindow) {
    window.close();
}

#[cfg(not(debug_assertions))]
fn handle_close_window(window: &gtk::ApplicationWindow) {
    window.set_visible(false);
}
