use std::path::PathBuf;

use crate::types::Kind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub kind: Kind,
    /// Identity: directory name (skills) or file stem (agents, md).
    pub name: String,
    pub description: Option<String>,
    /// The markdown file (SKILL.md for skills).
    pub path: PathBuf,
    pub size_bytes: u64,
}

impl Entry {
    /// Rough context cost: ~4 bytes per token.
    pub fn estimated_tokens(&self) -> u64 {
        self.size_bytes.div_ceil(4)
    }

    /// What gets linked into a session plugin: the directory for skills, the file otherwise.
    pub fn source(&self) -> PathBuf {
        match self.kind {
            Kind::Skill => self
                .path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| self.path.clone()),
            _ => self.path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: Kind, path: &str, size: u64) -> Entry {
        Entry {
            kind,
            name: "n".into(),
            description: None,
            path: path.into(),
            size_bytes: size,
        }
    }

    #[test]
    fn tokens_round_up() {
        assert_eq!(entry(Kind::Agent, "/a.md", 0).estimated_tokens(), 0);
        assert_eq!(entry(Kind::Agent, "/a.md", 1).estimated_tokens(), 1);
        assert_eq!(entry(Kind::Agent, "/a.md", 9).estimated_tokens(), 3);
    }

    #[test]
    fn source_is_dir_for_skills_file_otherwise() {
        assert_eq!(
            entry(Kind::Skill, "/l/s/SKILL.md", 1).source(),
            PathBuf::from("/l/s")
        );
        assert_eq!(
            entry(Kind::Agent, "/l/a.md", 1).source(),
            PathBuf::from("/l/a.md")
        );
    }
}
