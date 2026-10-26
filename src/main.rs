use gio::glib::property::PropertySet;
use gtk::{
    gio,
    glib::{self, subclass::types::ObjectSubclassIsExt},
    prelude::*,
};
use gtk4::{self as gtk};
use std::{fs, path};

mod constants;
mod model;
mod modules;
mod utils;
mod window;

use modules::config;
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

        let home_dir = glib::home_dir();
        let mut config_path = home_dir
            .join(constants::JOTTO_LIB_CONFIG_DIR)
            .join(constants::APP_CONFIG_DIR)
            .join(constants::CONF_FILE_NAME);

        let mut close = false;
        let mut term: Option<String> = None;
        let mut write_stdout = false;

        let mut i = 1;
        while i < args.len() {
            let arg = args[i].to_str().unwrap();
            match arg {
                "--close" => close = true,
                "--config" => {
                    let next = if i + 1 < args.len() {
                        path::PathBuf::from(&args[i + 1])
                    } else {
                        error_log!("Expected config file after \"--config\"");
                        return glib::ExitCode::FAILURE;
                    };

                    i += 1;

                    config_path = utils::resolve_home_relative_path(next, &home_dir);
                }
                "--term" => {
                    let next = if i + 1 < args.len() {
                        args[i + 1].to_str().unwrap().to_string()
                    } else {
                        error_log!("Expected config file after \"--config\"");
                        return glib::ExitCode::FAILURE;
                    };

                    i += 1;

                    term = Some(next);
                }
                "--write-stdout" => {
                    write_stdout = true;
                }
                other => error_log!(format!("Unrecognized arg: \"{}\"", other)),
            }

            i += 1;
        }

        let config_file = fs::read_to_string(config_path).map(Some).unwrap_or(None);
        let mut app_config = config_file
            .and_then(|file_data| config::parse_config(file_data, &home_dir))
            .unwrap_or(config::AppConfig::new(
                None,
                config::RenderPreset::None,
                vec![],
                false,
                false,
            ));

        if term.is_some() {
            app_config.term = term;
        }

        if write_stdout {
            app_config.write_stdout = true;
        }

        let window = match app.windows().first() {
            Some(win) => {
                let existing_win = win.clone().downcast::<SpotlightWindow>().unwrap();

                {
                    let imp = existing_win.imp();
                    imp.root_instance.set(false);
                    imp.app_config.set(app_config);
                    let app_config = imp.app_config.borrow();

                    let parse_config = parse_config_from_app_config(&app_config);
                    imp.config.set(parse_config);

                    let new_factory = window::build_factory(
                        app_config.render_preset,
                        window::ImageCache::new(constants::IMAGE_CACHE_CAP),
                    );

                    existing_win.set_list_factory(new_factory);
                    existing_win.run_search("");
                }

                existing_win
            }
            None => {
                utils::setup_icons();
                build_window(app, app_config, true)
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

fn build_window(
    app: &gtk::Application,
    app_config: config::AppConfig,
    is_root_instance: bool,
) -> SpotlightWindow {
    _ = app.hold();

    let parse_config = parse_config_from_app_config(&app_config);

    SpotlightWindow::new(app, app_config, parse_config, is_root_instance)
}

fn parse_config_from_app_config(app_config: &config::AppConfig) -> dir_search_rs::ParseConfig {
    let home_dir = glib::home_dir();
    let search_dirs: Vec<_> = app_config
        .search_dirs
        .clone()
        .into_iter()
        .map(|path| utils::resolve_home_relative_path(path, &home_dir))
        .collect();

    match app_config.render_preset {
        config::RenderPreset::DesktopFile => dir_search_rs::ParseConfig {
            search_dirs,
            search_strs: vec!["type=application".to_string(), "name={search}".to_string()],
            search_contents: dir_search_rs::SearchContents::FileContents(
                Some(vec![".desktop".to_string()]),
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
    }
}
