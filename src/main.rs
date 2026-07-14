use gio::glib::property::PropertySet;
use gtk::{
    gio,
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::*,
};
use gtk4 as gtk;
use std::{fmt, fs};

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

        let mut i = 1;
        while i < args.len() {
            let arg = args[i].to_str().unwrap();
            match arg {
                "--close" => close = true,
                "--config" => {
                    let next = if i + 1 < args.len() {
                        args[i + 1].to_str().unwrap().to_string()
                    } else {
                        error_log!("Expected config file after \"--config\"");
                        return glib::ExitCode::FAILURE;
                    };

                    i += 1;

                    config_path = Some(utils::resolve_home_relative_path(next, home_dir.as_ref()));
                }
                other => error_log!(format!("Unrecognized arg: \"{}\"", other)),
            }

            i += 1;
        }

        let config_file =
            config_path.and_then(|path| fs::read_to_string(path).map(Some).unwrap_or(None));
        let app_config = config_file
            .and_then(|file_data| utils::parse_config(file_data, home_dir.as_ref()))
            .unwrap_or(utils::AppConfig::new(
                None,
                utils::RenderPreset::None,
                vec![],
            ));

        let window = match app.windows().first() {
            Some(win) => {
                let existing_win = win.clone().downcast::<SpotlightWindow>().unwrap();

                {
                    let imp = existing_win.imp();
                    imp.app_config.set(app_config);
                    let app_config = imp.app_config.borrow();

                    if let Ok(parse_config) = parse_config_from_app_config(&app_config) {
                        _ = imp.config.set(parse_config);
                    }

                    let new_factory = window::build_factory(
                        app_config.render_preset,
                        window::ImageCache::new(constants::IMAGE_CACHE_CAP),
                    );

                    existing_win.set_list_factory(new_factory);
                    existing_win.run_search("");
                }

                existing_win
            }
            None => match build_window(app, app_config) {
                Ok(win) => win,
                Err(err) => {
                    error_log!(err);
                    return glib::ExitCode::FAILURE;
                }
            },
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
    _ = app.hold();

    let parse_config = parse_config_from_app_config(&app_config)?;

    Ok(SpotlightWindow::new(app, app_config, parse_config))
}

fn parse_config_from_app_config(
    app_config: &utils::AppConfig,
) -> Result<dir_search_rs::ParseConfig, WindowInitError> {
    let home_dir = utils::get_home_dir().ok_or(WindowInitError::CouldNotLocateHomeDir)?;
    let search_dirs: Vec<_> = app_config
        .search_dirs
        .clone()
        .into_iter()
        .map(|path| {
            utils::resolve_home_relative_path(path, Some(&home_dir.to_str().unwrap().to_string()))
        })
        .collect();

    let res = match app_config.render_preset {
        utils::RenderPreset::DesktopFile => dir_search_rs::ParseConfig {
            search_dirs,
            search_strs: vec!["type=application".to_string(), "name={search}".to_string()],
            search_contents: dir_search_rs::SearchContents::FileContents(
                Some(".desktop".to_string()),
                true,
            ),
            parallel_preference: None,
        },
        _ => dir_search_rs::ParseConfig {
            search_dirs,
            search_strs: vec!["{search}".to_string()],
            search_contents: dir_search_rs::SearchContents::FileName(false),
            parallel_preference: None,
        },
    };

    Ok(res)
}
