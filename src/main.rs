use gtk::{
    gio,
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::*,
};
use gtk4 as gtk;
use std::{fmt, fs, rc::Rc};

mod constants;
mod model;
mod modules;
mod utils;
mod window;

use window::SpotlightWindow;

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("com.jackson.spotlight")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|app| {
        utils::load_css();

        app.set_accels_for_action("win.close", &["Escape"]);
        app.set_accels_for_action("win.select-prev", &["Up"]);
        app.set_accels_for_action("win.select-next", &["Down"]);
        app.set_accels_for_action("win.launch-selected", &["Return", "KP_Enter"]);
    });

    app.connect_command_line(move |app, cmd_line| {
        let args: Vec<_> = cmd_line.arguments();

        let home_dir = utils::get_home_dir().map(|dir| dir.to_str().unwrap().to_string());
        let mut config_path = home_dir.as_ref().map(|dir| {
            format!(
                "{}/.config/{}/{}/{}",
                dir,
                constants::JOTTO_LIB_CONFIG_DIR,
                constants::APP_CONFIG_DIR,
                constants::CONF_FILE_NAME
            )
        });

        let mut close = false;
        let mut render_preset: Option<utils::RenderPreset> = None;
        let mut term: Option<String> = None;

        let mut i = 1;
        while i < args.len() {
            let arg = args[i].to_str().unwrap();
            match arg {
                "--close" => close = true,
                "--render" => {
                    let next = if i + 1 < args.len() {
                        args[i + 1].to_str().unwrap().to_string()
                    } else {
                        error_log!("Expected render type after \"--render\"");
                        return glib::ExitCode::FAILURE;
                    };

                    i += 1;

                    if render_preset.is_some() {
                        error_log_exit!("Duplicate render preset options found");
                    }

                    render_preset = utils::RenderPreset::from_str(&next);
                }
                "--config" => {
                    let next = if i + 1 < args.len() {
                        args[i + 1].to_str().unwrap().to_string()
                    } else {
                        error_log!("Expected config file after \"--config\"");
                        return glib::ExitCode::FAILURE;
                    };

                    i += 1;

                    config_path = if next.starts_with("~")
                        && let Some(dir) = &home_dir
                    {
                        Some(next.replace("~", dir))
                    } else {
                        Some(next.to_string())
                    };
                }
                "--term" => {
                    let next = if i + 1 < args.len() {
                        args[i + 1].to_str().unwrap().to_string()
                    } else {
                        error_log!("Expected config file after \"--config\"");
                        return glib::ExitCode::FAILURE;
                    };

                    i += 1;

                    term = Some(next.to_string());
                }
                other => error_log!(format!("Unrecognized arg: \"{}\"", other)),
            }

            i += 1;
        }

        let config_file =
            config_path.and_then(|path| fs::read_to_string(path).map(Some).unwrap_or(None));
        let mut app_config = config_file
            .map(utils::parse_config)
            .unwrap_or(utils::AppConfig::new(None, render_preset));

        if term.is_some() {
            app_config.term = term;
        }

        if render_preset.is_some() {
            app_config.render_preset = render_preset;
        }

        let window = match app.windows().first() {
            Some(win) => win.clone().downcast::<SpotlightWindow>().unwrap(),
            None => {
                if app_config.render_preset.is_none() {
                    error_log_exit!("Expected render preset");
                };

                match build_window(app, app_config) {
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

        if close {
            window.set_visible(false);
        }

        let window_imp = window.imp();
        window_imp.cli_connection.set(Some(cmd_line.clone()));

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
    app_config: utils::AppConfig,
) -> Result<SpotlightWindow, WindowInitError> {
    let Some(render_preset) = app_config.render_preset else {
        error_log_exit!("Expected render preset at build window");
    };

    _ = app.hold();

    let home_dir = utils::get_home_dir().ok_or(WindowInitError::CouldNotLocateHomeDir)?;

    let parse_config = match render_preset {
        utils::RenderPreset::DesktopFile => dir_search_rs::ParseConfig {
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
        },
        _ => dir_search_rs::ParseConfig {
            search_dirs: vec![utils::prefix_path_str(home_dir, "wallpapers")],
            search_strs: vec!["{search}".to_string()],
            search_contents: dir_search_rs::SearchContents::FileName(false),
            parallel_preference: None,
        },
    };
    let parse_config = Rc::new(parse_config);

    Ok(SpotlightWindow::new(app, app_config, parse_config))
}
