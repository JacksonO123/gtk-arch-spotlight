use gtk::{gdk, gio, glib};
use gtk4::{self as gtk, IconTheme, Settings};
use std::path;

use crate::constants::{APP_CONFIG_DIR, DEFAULT_STYLES, JOTTO_LIB_CONFIG_DIR, STYLE_FILE};

#[macro_export]
macro_rules! error_fmt {
    ($arg:expr) => {
        format!("[ERROR]: {}", $arg)
    };
}

#[macro_export]
macro_rules! error_log {
    ($arg:expr) => {
        eprintln!("{}", error_fmt!($arg))
    };
}

#[macro_export]
macro_rules! error_log_exit {
    ($arg:expr) => {
        error_log!($arg);
        std::process::exit(1);
    };
}

pub fn load_css() {
    let default_display = &gdk::Display::default().expect("Could not connect to a display");

    let defaults = gtk::CssProvider::new();
    defaults.load_from_string(DEFAULT_STYLES);
    gtk::style_context_add_provider_for_display(
        default_display,
        &defaults,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let mut config_path = glib::user_config_dir();
    config_path.push(JOTTO_LIB_CONFIG_DIR);
    config_path.push(APP_CONFIG_DIR);
    config_path.push(STYLE_FILE);

    if config_path.exists() {
        let user = gtk::CssProvider::new();
        let gio_file = gio::File::for_path(config_path);
        user.load_from_file(&gio_file);
        gtk::style_context_add_provider_for_display(
            default_display,
            &user,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

pub fn resolve_home_relative_path(
    rel_path: path::PathBuf,
    home_dir: &path::PathBuf,
) -> path::PathBuf {
    if rel_path.starts_with("~/") {
        let new_path: path::PathBuf = rel_path
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .skip(1)
            .collect();
        return home_dir.join(new_path);
    }

    rel_path
}

pub fn setup_icons() {
    let display = gdk::Display::default().expect("no display");

    let settings = Settings::for_display(&display);
    settings.set_gtk_icon_theme_name(Some("crystal-remix-icon-theme"));

    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_search_path(glib::user_data_dir().join("icons"));

    println!("has icon: {}", icon_theme.has_icon("your-icon-name"));
    println!("search path: {:?}", icon_theme.search_path());
}
