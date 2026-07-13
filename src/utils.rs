use gtk::{gdk, gio, glib};
use gtk4 as gtk;
use std::{env, path};

use crate::constants::{APP_CONFIG_DIR, DEFAULT_STYLES, JOTTO_LIB_CONFIG_DIR, STYLE_FILE};

#[macro_export]
macro_rules! error_log {
    ($arg:expr) => {
        eprintln!("[ERROR]: {}", $arg)
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

#[derive(Debug, Clone, Copy)]
pub enum RenderPreset {
    DesktopFile,
    Images,
}

impl RenderPreset {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "desktop-file" => Some(Self::DesktopFile),
            "image" => Some(Self::Images),
            _ => {
                error_log!(format!(
                    "Unexpected render preset value \"{}\" expected one of (\"desktop-file\", \"image\")",
                    str
                ));
                None
            }
        }
    }
}

pub fn get_home_dir() -> Option<path::PathBuf> {
    #[cfg(not(unix))]
    {
        panic!("Unsupported os. I hope you are not using windows.");
    }

    env::var_os("HOME").map(path::PathBuf::from)
}

pub fn prefix_path_str(dir_path: path::PathBuf, path: &str) -> String {
    let mut home_clone = dir_path.clone();
    home_clone.push(path);
    home_clone.to_str().unwrap().to_string()
}

#[derive(Debug)]
pub struct AppConfig {
    pub term: Option<String>,
    pub render_preset: Option<RenderPreset>,
}

impl AppConfig {
    pub fn new(term: Option<String>, render_preset: Option<RenderPreset>) -> Self {
        Self {
            term,
            render_preset,
        }
    }
}

pub fn parse_config(config_file: String) -> AppConfig {
    let lines = config_file.split('\n');
    let mut term: Option<String> = None;
    let mut render_preset: Option<RenderPreset> = None;

    for line in lines {
        let parts: Vec<_> = line.splitn(2, '=').collect();
        if parts.len() < 2 {
            if parts[0].len() > 0 {
                error_log!(format!("Expected \"=\" after {}", parts[0]));
            }
            continue;
        }
        if parts[1].len() == 0 {
            error_log!(format!("Expected value after {}=", parts[0]));
        }

        match parts[0] {
            "term" => term = Some(parts[1].to_string()),
            "render_preset" => {
                RenderPreset::from_str(parts[1]).inspect(|val| render_preset = Some(val.clone()));
            }
            _ => {
                error_log!(format!("Unexpected config key {}", parts[0]))
            }
        }
    }

    AppConfig::new(term, render_preset)
}
