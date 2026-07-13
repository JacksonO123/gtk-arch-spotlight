use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A parsed `.desktop` application entry.
///
/// This is the plain-data representation of a launcher result. Each file is
/// read and parsed exactly once into this struct; everything the UI needs
/// (display name, icon, launch command) is derived from here.
#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: Option<String>,
    pub icon: Option<String>,
    #[allow(dead_code)]
    pub path: PathBuf,
}

impl DesktopEntry {
    /// Parse a `.desktop` file, reading only the `[Desktop Entry]` group.
    ///
    /// Returns `None` when the file can't be read, has no `Name`, or is marked
    /// `NoDisplay`/`Hidden` (entries that should never appear in a launcher).
    pub fn from_path(path: &Path) -> Option<Self> {
        let contents = fs::read_to_string(path).ok()?;

        let mut in_entry = false;
        let mut name: Option<String> = None;
        let mut exec: Option<String> = None;
        let mut icon: Option<String> = None;
        let mut no_display = false;
        let mut hidden = false;

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

            // Only the plain keys — localized variants like `Name[de]` are
            // intentionally ignored, and the first occurrence wins.
            match key.trim() {
                "Name" if name.is_none() => name = Some(value.to_string()),
                "Exec" if exec.is_none() => exec = Some(value.to_string()),
                "Icon" if icon.is_none() => icon = Some(value.to_string()),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
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
        })
    }

    /// Launch the application described by this entry's `Exec` line.
    ///
    /// Field codes (`%f`, `%U`, `%i`, …) are stripped per the Desktop Entry
    /// spec; the child is spawned detached and not waited on.
    pub fn launch(&self) -> io::Result<()> {
        let exec = self
            .exec
            .as_deref()
            .ok_or_else(|| io::Error::other("desktop entry has no Exec key"))?;

        let mut tokens = exec.split_whitespace().filter_map(|token| {
            if token == "%%" {
                Some("%".to_string())
            } else if token.starts_with('%') {
                // Standalone field code (e.g. `%U`, `%f`) — drop it.
                None
            } else {
                Some(token.to_string())
            }
        });

        let program = tokens
            .next()
            .ok_or_else(|| io::Error::other("desktop entry Exec is empty"))?;

        Command::new(program).args(tokens).spawn().map(|_| ())
    }
}
