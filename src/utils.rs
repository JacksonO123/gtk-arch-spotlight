use gtk::{gdk, gio, glib};
use gtk4 as gtk;
use std::{env, path};

use crate::constants::{self, APP_CONFIG_DIR, DEFAULT_STYLES, JOTTO_LIB_CONFIG_DIR, STYLE_FILE};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderPreset {
    DesktopFile,
    Images,
    None,
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

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "AppConfig")]
pub struct AppConfig {
    pub term: Option<String>,
    pub render_preset: RenderPreset,
    pub search_dirs: Vec<String>,
}

impl AppConfig {
    pub fn new(
        term: Option<String>,
        render_preset: RenderPreset,
        search_dirs: Vec<String>,
    ) -> Self {
        Self {
            term,
            render_preset,
            search_dirs,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            term: Default::default(),
            render_preset: RenderPreset::None,
            search_dirs: Default::default(),
        }
    }
}

pub fn parse_config(config_file: String, home_dir: Option<&String>) -> Option<AppConfig> {
    let lines: Vec<_> = config_file.split('\n').collect();
    let mut term: Option<String> = None;
    let mut render_preset: Option<RenderPreset> = None;
    let mut search_dirs: Vec<String> = vec![];

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        if line == constants::DIR_START_SENTINEL {
            if i + 1 < lines.len() {
                i += 1;
            } else {
                break;
            }

            let mut current_path = lines[i];
            while i < lines.len() && current_path != constants::DIR_END_SENTINEL {
                search_dirs.push(resolve_home_relative_path(
                    current_path.to_string(),
                    home_dir,
                ));

                i += 1;
                current_path = lines[i];
            }

            i += 1;
            continue;
        }

        let parts: Vec<_> = line.splitn(2, '=').collect();
        if parts.len() < 2 {
            if !parts[0].is_empty() {
                error_log!(format!("Expected \"=\" after {}", parts[0]));
            }
            i += 1;
            continue;
        }
        if parts[1].is_empty() {
            error_log!(format!("Expected value after {}=", parts[0]));
        }

        let value = parts[1];

        match parts[0] {
            "term" => term = Some(value.to_string()),
            "render_preset" => {
                RenderPreset::from_str(value).inspect(|val| render_preset = Some(*val));
            }
            _ => {
                error_log!(format!("Unexpected config key {}", parts[0]))
            }
        }

        i += 1;
    }

    render_preset.map(|preset| AppConfig::new(term, preset, search_dirs))
}

pub fn resolve_home_relative_path(path: String, home_dir: Option<&String>) -> String {
    if path.starts_with("~")
        && let Some(dir) = &home_dir
    {
        return path.replacen("~", dir, 1);
    }

    path
}
