use std::fs;
use std::path::Path;

use super::entry::Entry;
use super::frontmatter;
use crate::error::{BcError, Result};
use crate::types::Kind;

#[derive(Clone, Debug, Default)]
pub struct Library {
    entries: Vec<Entry>,
}

impl Library {
    pub fn from_entries(mut entries: Vec<Entry>) -> Self {
        entries.sort_by(|a, b| (a.kind, &a.name).cmp(&(b.kind, &b.name)));
        Self { entries }
    }

    /// Scans `<library_dir>/{skills,agents,claude-md}`; missing dirs are empty.
    pub fn scan(library_dir: &Path) -> Result<Self> {
        let mut entries = Vec::new();
        for kind in Kind::ALL {
            scan_kind(&library_dir.join(kind.subdir()), kind, &mut entries)?;
        }
        Ok(Self::from_entries(entries))
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn of_kind(&self, kind: Kind) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(move |e| e.kind == kind)
    }

    pub fn find(&self, kind: Kind, name: &str) -> Option<&Entry> {
        self.of_kind(kind).find(|e| e.name == name)
    }
}

fn scan_kind(dir: &Path, kind: Kind, out: &mut Vec<Entry>) -> Result<()> {
    let read = match fs::read_dir(dir) {
        Ok(read) => read,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(BcError::io(dir, err)),
    };
    for item in read {
        let item = item.map_err(|e| BcError::io(dir, e))?;
        let path = item.path();
        let (name, file) = match kind {
            Kind::Skill if path.is_dir() => (file_name(&path), path.join("SKILL.md")),
            Kind::Agent | Kind::ClaudeMd
                if path.is_file() && path.extension().is_some_and(|e| e == "md") =>
            {
                (stem(&path), path.clone())
            }
            _ => continue,
        };
        if !file.is_file() {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|e| BcError::io(&file, e))?;
        let size_bytes = fs::metadata(&file).map_err(|e| BcError::io(&file, e))?.len();
        out.push(Entry {
            kind,
            name,
            description: frontmatter::parse(&text).description,
            path: file,
            size_bytes,
        });
    }
    Ok(())
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn missing_library_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Library::scan(&dir.path().join("none")).unwrap().entries().is_empty());
    }

    #[test]
    fn scans_all_kinds_sorted_with_descriptions() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(&root.join("skills/zeta/SKILL.md"), "---\ndescription: Z skill\n---\nbody");
        write(&root.join("skills/alpha/SKILL.md"), "no frontmatter");
        write(&root.join("skills/empty/README.md"), "ignored: no SKILL.md");
        write(&root.join("agents/rev.md"), "---\ndescription: reviews\n---\n");
        write(&root.join("agents/notes.txt"), "ignored");
        write(&root.join("claude-md/personal.md"), "# me");

        let lib = Library::scan(root).unwrap();
        let names: Vec<_> = lib.entries().iter().map(|e| (e.kind, e.name.as_str())).collect();
        assert_eq!(
            names,
            [
                (Kind::Skill, "alpha"),
                (Kind::Skill, "zeta"),
                (Kind::Agent, "rev"),
                (Kind::ClaudeMd, "personal"),
            ]
        );
        assert_eq!(lib.find(Kind::Skill, "zeta").unwrap().description.as_deref(), Some("Z skill"));
        assert_eq!(lib.find(Kind::Skill, "alpha").unwrap().description, None);
        assert!(lib.find(Kind::Agent, "zeta").is_none());
        assert_eq!(lib.of_kind(Kind::Agent).count(), 1);
        assert_eq!(lib.find(Kind::ClaudeMd, "personal").unwrap().size_bytes, 4);
    }
}
