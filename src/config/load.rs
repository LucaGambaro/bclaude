use std::fs;
use std::path::Path;

use super::schema::Config;
use crate::error::{BcError, Result};

/// A missing file is an empty config; any other failure is an error.
pub fn load(path: &Path) -> Result<Config> {
    match fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).map_err(|source| BcError::ConfigParse {
            path: path.to_path_buf(),
            source,
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(err) => Err(BcError::io(path, err)),
    }
}

pub fn save(path: &Path, config: &Config) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| BcError::io(parent, e))?;
    }
    let text = toml::to_string_pretty(config)?;
    fs::write(path, text).map_err(|e| BcError::io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Kind;

    #[test]
    fn missing_file_gives_default() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load(&dir.path().join("nope.toml")).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn save_then_load_roundtrips_and_creates_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/config.toml");
        let mut cfg = Config::default();
        cfg.set_membership("global", Kind::Skill, "s", true);
        save(&path, &cfg).unwrap();
        assert_eq!(load(&path).unwrap(), cfg);
    }

    #[test]
    fn invalid_toml_is_a_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.toml");
        fs::write(&path, "profiles = [").unwrap();
        assert!(matches!(load(&path), Err(BcError::ConfigParse { .. })));
    }

    #[test]
    fn unreadable_path_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        // reading a directory as a file fails with something other than NotFound
        assert!(matches!(load(dir.path()), Err(BcError::Io { .. })));
    }
}
