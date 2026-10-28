use std::fs;
use std::path::Path;

use crate::error::{BcError, Result};
use crate::paths::Paths;
use crate::types::Kind;

/// Copies a file or directory recursively (symlinks are followed).
pub fn copy_recursive(from: &Path, to: &Path) -> Result<()> {
    if from.is_dir() {
        fs::create_dir_all(to).map_err(|e| BcError::io(to, e))?;
        for item in fs::read_dir(from).map_err(|e| BcError::io(from, e))? {
            let item = item.map_err(|e| BcError::io(from, e))?;
            copy_recursive(&item.path(), &to.join(item.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| BcError::io(parent, e))?;
        }
        fs::copy(from, to)
            .map(|_| ())
            .map_err(|e| BcError::io(to, e))
    }
}

/// Copies `source` into the library under `kind`. Skills accept the dir or its SKILL.md;
/// agents/md accept a `.md` file. Returns the entry name.
pub fn add(paths: &Paths, kind: Kind, source: &Path, name: Option<&str>) -> Result<String> {
    let invalid = |msg: String| BcError::Invalid(msg);
    let dir = paths.kind_dir(kind);
    match kind {
        Kind::Skill => {
            let skill_dir = if source.is_file() {
                source
                    .parent()
                    .ok_or_else(|| invalid(format!("no parent for {}", source.display())))?
            } else {
                source
            };
            if !skill_dir.join("SKILL.md").is_file() {
                return Err(invalid(format!("{} has no SKILL.md", skill_dir.display())));
            }
            let name = name
                .map(str::to_string)
                .or_else(|| {
                    skill_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                })
                .ok_or_else(|| invalid("cannot infer skill name".into()))?;
            copy_recursive(skill_dir, &dir.join(&name))?;
            Ok(name)
        }
        Kind::Agent | Kind::ClaudeMd => {
            if !source.is_file() || source.extension().is_none_or(|e| e != "md") {
                return Err(invalid(format!("{} is not a .md file", source.display())));
            }
            let name = name
                .map(str::to_string)
                .or_else(|| source.file_stem().map(|n| n.to_string_lossy().into_owned()))
                .ok_or_else(|| invalid("cannot infer name".into()))?;
            copy_recursive(source, &dir.join(format!("{name}.md")))?;
            Ok(name)
        }
    }
}

/// Imports skills/agents from `~/.claude` that are not yet in the library.
/// Returns `(kind, name)` of what was copied.
pub fn import_from_claude(paths: &Paths) -> Result<Vec<(Kind, String)>> {
    let mut imported = Vec::new();
    for kind in [Kind::Skill, Kind::Agent] {
        let src = paths.claude_dir().join(kind.subdir());
        let read = match fs::read_dir(&src) {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(BcError::io(&src, e)),
        };
        let mut candidates: Vec<_> = read.filter_map(|i| i.ok().map(|i| i.path())).collect();
        candidates.sort();
        for path in candidates {
            let valid = match kind {
                Kind::Skill => path.join("SKILL.md").is_file(),
                _ => path.is_file() && path.extension().is_some_and(|e| e == "md"),
            };
            if !valid {
                continue;
            }
            let stem = match kind {
                Kind::Skill => path.file_name(),
                _ => path.file_stem(),
            }
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
            let target = match kind {
                Kind::Skill => paths.kind_dir(kind).join(&stem),
                _ => paths.kind_dir(kind).join(format!("{stem}.md")),
            };
            if target.exists() {
                continue;
            }
            add(paths, kind, &path, None)?;
            imported.push((kind, stem));
        }
    }
    Ok(imported)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().join("bc"), dir.path().join("home"));
        (dir, paths)
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn adds_skill_from_dir_and_from_file_with_nested_files() {
        let (dir, paths) = setup();
        let src = dir.path().join("src/my-skill");
        write(&src.join("SKILL.md"), "x");
        write(&src.join("refs/a.txt"), "y");
        assert_eq!(add(&paths, Kind::Skill, &src, None).unwrap(), "my-skill");
        assert!(
            paths
                .kind_dir(Kind::Skill)
                .join("my-skill/refs/a.txt")
                .is_file()
        );
        assert_eq!(
            add(&paths, Kind::Skill, &src.join("SKILL.md"), Some("other")).unwrap(),
            "other"
        );
        assert!(paths.kind_dir(Kind::Skill).join("other/SKILL.md").is_file());
    }

    #[test]
    fn rejects_skill_without_skill_md_and_non_md_files() {
        let (dir, paths) = setup();
        let empty = dir.path().join("empty");
        fs::create_dir_all(&empty).unwrap();
        assert!(matches!(
            add(&paths, Kind::Skill, &empty, None),
            Err(BcError::Invalid(_))
        ));
        let txt = dir.path().join("a.txt");
        write(&txt, "x");
        assert!(matches!(
            add(&paths, Kind::Agent, &txt, None),
            Err(BcError::Invalid(_))
        ));
        assert!(matches!(
            add(&paths, Kind::ClaudeMd, &dir.path().join("missing.md"), None),
            Err(BcError::Invalid(_))
        ));
    }

    #[test]
    fn adds_agent_and_md_with_rename() {
        let (dir, paths) = setup();
        let f = dir.path().join("rev.md");
        write(&f, "x");
        assert_eq!(add(&paths, Kind::Agent, &f, None).unwrap(), "rev");
        assert_eq!(
            add(&paths, Kind::ClaudeMd, &f, Some("shared")).unwrap(),
            "shared"
        );
        assert!(paths.kind_dir(Kind::Agent).join("rev.md").is_file());
        assert!(paths.kind_dir(Kind::ClaudeMd).join("shared.md").is_file());
    }

    #[test]
    fn import_copies_new_items_only() {
        let (_dir, paths) = setup();
        let claude = paths.claude_dir();
        write(&claude.join("skills/s1/SKILL.md"), "a");
        write(&claude.join("skills/not-a-skill/readme.md"), "a");
        write(&claude.join("agents/a1.md"), "a");
        write(&claude.join("agents/a2.md"), "a");
        write(&paths.kind_dir(Kind::Agent).join("a2.md"), "already");

        let done = import_from_claude(&paths).unwrap();
        assert_eq!(
            done,
            [
                (Kind::Skill, "s1".to_string()),
                (Kind::Agent, "a1".to_string())
            ]
        );
        assert_eq!(
            fs::read_to_string(paths.kind_dir(Kind::Agent).join("a2.md")).unwrap(),
            "already"
        );
        assert!(import_from_claude(&paths).unwrap().is_empty());
    }

    #[test]
    fn import_without_claude_dir_is_noop() {
        let (_dir, paths) = setup();
        assert!(import_from_claude(&paths).unwrap().is_empty());
    }
}
