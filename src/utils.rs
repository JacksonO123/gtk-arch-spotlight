use gtk::{gdk, gio, glib};
use gtk4 as gtk;
use std::{env, path};

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

pub fn get_home_dir() -> Option<path::PathBuf> {
    #[cfg(not(unix))]
    {
        panic!("Unsupported os. I hope you are not using windows.");
    }

    env::var_os("HOME").map(path::PathBuf::from)
}

pub fn resolve_home_relative_path(path: String, home_dir: Option<&String>) -> String {
    if path.starts_with("~")
        && let Some(dir) = &home_dir
    {
        return path.replacen("~", dir, 1);
    }

    path
}
