use crate::rules::Progress;
use std::path::Path;

/// Why a save could not be read or written.
#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    /// Le fichier existe mais ne dit pas ce qu'il devrait.
    Malformed(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read the save: {e}"),
            Self::Malformed(what) => write!(f, "the save is malformed: {what}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Writes progress as TOML.
///
/// Distinct d'une scene : une scene decrit le monde tel qu'ecrit, une
/// sauvegarde ce que la partie en a fait.
#[must_use]
pub fn to_toml(progress: &Progress) -> String {
    let cleared: Vec<String> = progress.cleared.iter().map(usize::to_string).collect();

    format!(
        "[progress]\nhealth = {}\nkeys = {}\nroom = {}\ncleared = [{}]\n",
        progress.health,
        progress.keys,
        progress.room,
        cleared.join(", ")
    )
}

/// Reads progress back.
///
/// # Errors
///
/// If a field is missing or is not a number.
pub fn from_toml(text: &str) -> Result<Progress, SaveError> {
    let mut progress = Progress::new();
    let mut seen = false;

    for line in text.lines() {
        let line = line.trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());

        match key {
            "health" => progress.health = number(value, key)?,
            "keys" => progress.keys = number(value, key)?,
            "room" => progress.room = number(value, key)? as usize,
            "cleared" => {
                progress.cleared = value
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        s.parse::<usize>()
                            .map_err(|_| SaveError::Malformed(format!("`{s}` is not a room")))
                    })
                    .collect::<Result<_, _>>()?;
            }
            _ => continue,
        }
        seen = true;
    }

    if !seen {
        return Err(SaveError::Malformed("no fields at all".to_owned()));
    }

    // Une sauvegarde trafiquee ne doit pas donner un joueur invincible.
    progress.health = progress.health.min(crate::rules::MAX_HEALTH);

    Ok(progress)
}

fn number(value: &str, key: &str) -> Result<u32, SaveError> {
    value
        .parse()
        .map_err(|_| SaveError::Malformed(format!("`{key}` is not a number: {value}")))
}

/// # Errors
///
/// If the file cannot be written.
pub fn save(progress: &Progress, path: impl AsRef<Path>) -> Result<(), SaveError> {
    std::fs::write(path, to_toml(progress))?;
    Ok(())
}

/// # Errors
///
/// If the file cannot be read, or is malformed.
pub fn load(path: impl AsRef<Path>) -> Result<Progress, SaveError> {
    from_toml(&std::fs::read_to_string(path)?)
}

/// Where a save lives, beside the executable's data directory.
#[must_use]
pub fn default_path() -> std::path::PathBuf {
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(".keystone-save.toml")
}
