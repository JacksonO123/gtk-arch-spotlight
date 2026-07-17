use gio::glib;

use crate::{constants, error_fmt, error_log, utils};

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

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "AppConfig")]
pub struct AppConfig {
    pub term: Option<String>,
    pub render_preset: RenderPreset,
    pub search_dirs: Vec<String>,
    pub write_stdout: bool,
}

impl AppConfig {
    pub fn new(
        term: Option<String>,
        render_preset: RenderPreset,
        search_dirs: Vec<String>,
        write_stdout: bool,
    ) -> Self {
        Self {
            term,
            render_preset,
            search_dirs,
            write_stdout,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            term: None,
            render_preset: RenderPreset::None,
            search_dirs: Vec::new(),
            write_stdout: false,
        }
    }
}

pub fn parse_config(config_file: String, home_dir: Option<&String>) -> Option<AppConfig> {
    let lines: Vec<_> = config_file.split('\n').collect();
    let mut term: Option<String> = None;
    let mut render_preset: Option<RenderPreset> = None;
    let mut search_dirs: Vec<String> = vec![];
    let mut write_stdout = false;

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
                search_dirs.push(utils::resolve_home_relative_path(
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
            "write_stdout" => write_stdout = value == "true",
            _ => {
                error_log!(format!("Unexpected config key {}", parts[0]))
            }
        }

        i += 1;
    }

    render_preset.map(|preset| AppConfig::new(term, preset, search_dirs, write_stdout))
}
