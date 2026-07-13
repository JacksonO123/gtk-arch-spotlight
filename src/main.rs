use gtk::prelude::*;
use gtk::{gio, glib};
use gtk4 as gtk;
use std::{fmt, rc::Rc};

mod constants;
mod model;
mod modules;
mod utils;
mod window;

use utils::RenderPreset;
use window::SpotlightWindow;

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("com.jackson.spotlight")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|app| {
        utils::load_css();

        // Keyboard handling is expressed as window actions; the accelerators
        // route the relevant keys to them. They fire in the capture phase, so
        // they work even while the search entry holds focus.
        app.set_accels_for_action("win.close", &["Escape"]);
        app.set_accels_for_action("win.select-prev", &["Up"]);
        app.set_accels_for_action("win.select-next", &["Down"]);
        app.set_accels_for_action("win.launch-selected", &["Return", "KP_Enter"]);
    });

    app.connect_command_line(move |app, cmd_line| {
        let args: Vec<_> = cmd_line.arguments();

        let mut config = utils::AppConfig {
            close: false,
            render_preset: None,
        };

        for arg in args.iter().skip(1) {
            match arg.to_str().unwrap() {
                "--close" => config.close = true,
                "--render-desktop-files" => {
                    if config.render_preset.is_some() {
                        error_log_exit!("Duplicate render preset options found");
                    }
                    config.render_preset = Some(RenderPreset::DesktopFile);
                }
                "--render-images" => {
                    if config.render_preset.is_some() {
                        error_log_exit!("Duplicate render preset options found");
                    }
                    config.render_preset = Some(RenderPreset::Images);
                }
                other => error_log!(format!("Unrecognized arg: \"{}\"", other)),
            }
        }

        let window = match app.windows().first() {
            Some(win) => win.clone().downcast::<SpotlightWindow>().unwrap(),
            None => {
                let Some(render_preset) = config.render_preset else {
                    error_log_exit!("Expected render preset");
                };

                match build_window(app, render_preset) {
                    Ok(win) => win,
                    Err(err) => {
                        error_log!(err);
                        return glib::ExitCode::FAILURE;
                    }
                }
            }
        };

        window.present();
        window.focus_entry();

        if config.close {
            window.set_visible(false);
        }

        glib::ExitCode::SUCCESS
    });

    app.run()
}

#[derive(Debug)]
enum WindowInitError {
    CouldNotLocateHomeDir,
}

impl fmt::Display for WindowInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_to_write = match self {
            WindowInitError::CouldNotLocateHomeDir => "could not locate home dir",
        };

        write!(f, "{}", str_to_write)
    }
}

fn build_window(
    app: &gtk::Application,
    render_preset: RenderPreset,
) -> Result<SpotlightWindow, WindowInitError> {
    // Keep the application alive for as long as the window exists.
    let _ = app.hold();

    let home_dir = utils::get_home_dir().ok_or(WindowInitError::CouldNotLocateHomeDir)?;

    let parse_config = Rc::new(dir_search_rs::ParseConfig {
        search_dirs: vec![
            "/usr/share/applications".to_string(),
            utils::prefix_path_str(home_dir, ".local/share/applications"),
            "/usr/local/share/applications".to_string(),
        ],
        search_strs: vec!["type=application".to_string(), "name={search}".to_string()],
        search_contents: dir_search_rs::SearchContents::FileContents(
            Some(".desktop".to_string()),
            true,
        ),
        parallel_preference: None,
    });

    Ok(SpotlightWindow::new(app, render_preset, parse_config))
}
