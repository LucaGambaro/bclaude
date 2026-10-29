use std::fs;
use std::path::Path;

use super::entry::Entry;
use super::plugin_schema::{InstalledPlugins, PluginManifest};
use crate::error::{BcError, Result};
use crate::types::Kind;

/// Plugins installed in Claude Code, read from its `installed_plugins.json` (the source of truth,
/// nothing is copied). A missing file means no plugins; a missing or unparsable manifest only
/// leaves the description empty.
pub fn scan(installed_file: &Path) -> Result<Vec<Entry>> {
    let text = match fs::read_to_string(installed_file) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(BcError::io(installed_file, e)),
    };
    let installed: InstalledPlugins =
        serde_json::from_str(&text).map_err(|source| BcError::InstalledPlugins {
            path: installed_file.to_path_buf(),
            source,
        })?;
    Ok(installed
        .plugins
        .into_iter()
        .filter_map(|(id, installs)| {
            let install = installs.into_iter().next()?;
            let path = Path::new(&install.install_path).to_path_buf();
            Some(Entry {
                kind: Kind::Plugin,
                name: id,
                description: manifest_description(&path),
                path,
                size_bytes: 0,
            })
        })
        .collect())
}

fn manifest_description(install_dir: &Path) -> Option<String> {
    let text = fs::read_to_string(install_dir.join(".claude-plugin/plugin.json")).ok()?;
    let manifest: PluginManifest = serde_json::from_str(&text).ok()?;
    manifest.description
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn missing_file_means_no_plugins() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scan(&dir.path().join("none.json")).unwrap().is_empty());
    }

    #[test]
    fn reads_installs_with_manifest_descriptions() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let alpha = root.join("cache/alpha");
        write(
            &alpha.join(".claude-plugin/plugin.json"),
            r#"{"description":"Alpha tools"}"#,
        );
        let broken = root.join("cache/broken");
        write(&broken.join(".claude-plugin/plugin.json"), "not json");
        let file = root.join("installed_plugins.json");
        write(
            &file,
            &format!(
                r#"{{"version":2,"plugins":{{
                    "b@mk":[{{"installPath":"{}"}}],
                    "a@mk":[{{"installPath":"{}"}},{{"installPath":"/other"}}],
                    "empty@mk":[]
                }}}}"#,
                broken.display(),
                alpha.display()
            ),
        );
        let entries = scan(&file).unwrap();
        let summary: Vec<_> = entries
            .iter()
            .map(|e| (e.kind, e.name.as_str(), e.description.as_deref()))
            .collect();
        assert_eq!(
            summary,
            [
                (Kind::Plugin, "a@mk", Some("Alpha tools")),
                (Kind::Plugin, "b@mk", None)
            ]
        );
        assert_eq!(entries[0].path, alpha);
        assert_eq!(entries[0].size_bytes, 0);
    }

    #[test]
    fn invalid_json_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("installed_plugins.json");
        write(&file, "{");
        assert!(matches!(scan(&file), Err(BcError::InstalledPlugins { .. })));
    }

    #[test]
    fn unreadable_path_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(scan(dir.path()), Err(BcError::Io { .. })));
    }
}
