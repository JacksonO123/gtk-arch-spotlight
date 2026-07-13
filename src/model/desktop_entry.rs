use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: Option<String>,
    pub icon: Option<String>,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub terminal: bool,
}

impl DesktopEntry {
    pub fn from_path(path: &Path) -> Option<Self> {
        let contents = fs::read_to_string(path).ok()?;

        let mut in_entry = false;
        let mut name: Option<String> = None;
        let mut exec: Option<String> = None;
        let mut icon: Option<String> = None;
        let mut no_display = false;
        let mut hidden = false;
        let mut terminal = false;

        for line in contents.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                in_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_entry {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();

            match key.trim() {
                "Name" if name.is_none() => name = Some(value.to_string()),
                "Exec" if exec.is_none() => exec = Some(value.to_string()),
                "Icon" if icon.is_none() => icon = Some(value.to_string()),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                "Terminal" => {
                    terminal = match value {
                        "true" => true,
                        "false" => false,
                        _ => {
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }

        if no_display || hidden {
            return None;
        }

        Some(Self {
            name: name?,
            exec,
            icon,
            path: path.to_path_buf(),
            terminal,
        })
    }

    pub fn launch(&self, term_exec: Option<&str>) -> io::Result<()> {
        let exec = self
            .exec
            .as_deref()
            .ok_or_else(|| io::Error::other("desktop entry has no Exec key"))?;

        let exec = if self.terminal
            && let Some(term_exec) = term_exec
        {
            format!("{} {}", term_exec, exec)
        } else {
            exec.to_string()
        };

        let mut tokens = exec.split_whitespace().filter_map(|token| {
            if token == "%%" {
                Some("%".to_string())
            } else if token.starts_with('%') {
                None
            } else {
                Some(token.to_string())
            }
        });

        let program = tokens
            .next()
            .ok_or_else(|| io::Error::other("desktop entry Exec is empty"))?;

        Command::new(program)
            .args(tokens)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .map(|_| ())
    }
}
