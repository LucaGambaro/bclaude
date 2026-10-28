use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::error::{BcError, Result};
use crate::library::Library;
use crate::resolve::Resolved;
use crate::types::Kind;

pub const PLUGIN_NAME: &str = "bc";

/// Artifacts generated for one session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composed {
    /// Plugin dir with skills/agents; `None` when the session has neither.
    pub plugin_dir: Option<PathBuf>,
    /// Concatenated CLAUDE.md fragments; `None` when there are none.
    pub claude_md: Option<PathBuf>,
    /// Names referenced by the config but absent from the library.
    pub missing: Vec<(Kind, String)>,
}

/// Builds `<session_dir>/plugin` (symlinks into the library) and `<session_dir>/CLAUDE.md`.
pub fn compose(resolved: &Resolved, library: &Library, session_dir: &Path) -> Result<Composed> {
    let mut missing = Vec::new();
    let plugin = session_dir.join("plugin");
    let mut has_plugin_content = false;

    for kind in [Kind::Skill, Kind::Agent] {
        for name in resolved.names(kind) {
            let Some(entry) = library.find(kind, name) else {
                missing.push((kind, name.clone()));
                continue;
            };
            let target = match kind {
                Kind::Skill => plugin.join("skills").join(name),
                _ => plugin.join("agents").join(format!("{name}.md")),
            };
            link(&entry.source(), &target)?;
            has_plugin_content = true;
        }
    }
    if has_plugin_content {
        let manifest_dir = plugin.join(".claude-plugin");
        fs::create_dir_all(&manifest_dir).map_err(|e| BcError::io(&manifest_dir, e))?;
        let manifest = manifest_dir.join("plugin.json");
        let json = format!(
            "{{\n  \"name\": \"{PLUGIN_NAME}\",\n  \"version\": \"0.1.0\",\n  \"description\": \"Skills and agents composed by betterClaude\"\n}}\n"
        );
        fs::write(&manifest, json).map_err(|e| BcError::io(&manifest, e))?;
    }

    let mut sections = Vec::new();
    for name in &resolved.claude_md {
        match library.find(Kind::ClaudeMd, name) {
            Some(entry) => {
                let text =
                    fs::read_to_string(&entry.path).map_err(|e| BcError::io(&entry.path, e))?;
                sections.push(format!("<!-- bc: {name} -->\n{}\n", text.trim_end()));
            }
            None => missing.push((Kind::ClaudeMd, name.clone())),
        }
    }
    let claude_md = if sections.is_empty() {
        None
    } else {
        let file = session_dir.join("CLAUDE.md");
        fs::create_dir_all(session_dir).map_err(|e| BcError::io(session_dir, e))?;
        fs::write(&file, sections.join("\n")).map_err(|e| BcError::io(&file, e))?;
        Some(file)
    };

    Ok(Composed {
        plugin_dir: has_plugin_content.then_some(plugin),
        claude_md,
        missing,
    })
}

fn link(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| BcError::io(parent, e))?;
    }
    symlink(source, target).map_err(|e| BcError::io(target, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Entry;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn library(root: &Path) -> Library {
        write(&root.join("skills/s1/SKILL.md"), "skill one");
        write(&root.join("agents/a1.md"), "agent one");
        write(&root.join("claude-md/m1.md"), "first\n");
        write(&root.join("claude-md/m2.md"), "second");
        Library::scan(root).unwrap()
    }

    fn resolved(skills: &[&str], agents: &[&str], md: &[&str]) -> Resolved {
        let own = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
        Resolved {
            activations: vec![],
            skills: own(skills),
            agents: own(agents),
            claude_md: own(md),
        }
    }

    #[test]
    fn links_skills_and_agents_and_writes_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let lib = library(&dir.path().join("lib"));
        let session = dir.path().join("run/1");
        let c = compose(&resolved(&["s1"], &["a1"], &[]), &lib, &session).unwrap();
        let plugin = c.plugin_dir.unwrap();
        assert_eq!(
            fs::read_to_string(plugin.join("skills/s1/SKILL.md")).unwrap(),
            "skill one"
        );
        assert_eq!(
            fs::read_to_string(plugin.join("agents/a1.md")).unwrap(),
            "agent one"
        );
        assert!(
            fs::read_to_string(plugin.join(".claude-plugin/plugin.json"))
                .unwrap()
                .contains("\"bc\"")
        );
        assert!(c.claude_md.is_none());
        assert!(c.missing.is_empty());
    }

    #[test]
    fn concatenates_claude_md_in_order_with_headers() {
        let dir = tempfile::tempdir().unwrap();
        let lib = library(&dir.path().join("lib"));
        let c = compose(
            &resolved(&[], &[], &["m2", "m1"]),
            &lib,
            &dir.path().join("s"),
        )
        .unwrap();
        assert!(c.plugin_dir.is_none());
        let text = fs::read_to_string(c.claude_md.unwrap()).unwrap();
        assert_eq!(text, "<!-- bc: m2 -->\nsecond\n\n<!-- bc: m1 -->\nfirst\n");
    }

    #[test]
    fn reports_missing_entries_without_failing() {
        let dir = tempfile::tempdir().unwrap();
        let lib = library(&dir.path().join("lib"));
        let c = compose(
            &resolved(&["ghost"], &["a1", "nope"], &["zzz"]),
            &lib,
            &dir.path().join("s"),
        )
        .unwrap();
        assert_eq!(
            c.missing,
            [
                (Kind::Skill, "ghost".into()),
                (Kind::Agent, "nope".into()),
                (Kind::ClaudeMd, "zzz".into())
            ]
        );
        assert!(c.plugin_dir.is_some());
        assert!(c.claude_md.is_none());
    }

    #[test]
    fn nothing_resolved_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("s");
        let c = compose(&Resolved::default(), &Library::default(), &session).unwrap();
        assert_eq!(
            c,
            Composed {
                plugin_dir: None,
                claude_md: None,
                missing: vec![]
            }
        );
        assert!(!session.exists());
    }

    #[test]
    fn fails_when_target_already_linked() {
        let dir = tempfile::tempdir().unwrap();
        let lib = library(&dir.path().join("lib"));
        let session = dir.path().join("s");
        let r = resolved(&["s1"], &[], &[]);
        compose(&r, &lib, &session).unwrap();
        assert!(matches!(
            compose(&r, &lib, &session),
            Err(BcError::Io { .. })
        ));
        let _ = Entry {
            kind: Kind::Skill,
            name: String::new(),
            description: None,
            path: PathBuf::new(),
            size_bytes: 0,
        };
    }
}
