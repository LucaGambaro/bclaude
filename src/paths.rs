use std::path::{Path, PathBuf};

use crate::error::{BcError, Result};
use crate::types::Kind;

/// Filesystem locations used by betterClaude.
#[derive(Clone, Debug)]
pub struct Paths {
    pub root: PathBuf,
    pub home: PathBuf,
}

impl Paths {
    /// `BC_HOME` overrides the root; otherwise `$HOME/.betterclaude`.
    pub fn from_env() -> Result<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        match (std::env::var_os("BC_HOME"), home) {
            (Some(root), home) => Ok(Self::new(root.into(), home.unwrap_or_default())),
            (None, Some(home)) => Ok(Self::new(home.join(".betterclaude"), home)),
            (None, None) => Err(BcError::HomeNotSet),
        }
    }

    pub fn new(root: PathBuf, home: PathBuf) -> Self {
        Self { root, home }
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn library_dir(&self) -> PathBuf {
        self.root.join("library")
    }

    pub fn kind_dir(&self, kind: Kind) -> PathBuf {
        self.library_dir().join(kind.subdir())
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    pub fn run_dir(&self) -> PathBuf {
        self.root.join("run")
    }

    /// Where the user's own Claude config lives (source for `bc import`).
    pub fn claude_dir(&self) -> PathBuf {
        self.home.join(".claude")
    }
}

/// Expands a leading `~` (alone or followed by `/`) using `home`.
pub fn expand_tilde(raw: &str, home: &Path) -> PathBuf {
    if raw == "~" {
        home.to_path_buf()
    } else if let Some(rest) = raw.strip_prefix("~/") {
        home.join(rest)
    } else {
        PathBuf::from(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tilde_forms() {
        let home = Path::new("/h");
        assert_eq!(expand_tilde("~", home), PathBuf::from("/h"));
        assert_eq!(expand_tilde("~/a/b", home), PathBuf::from("/h/a/b"));
        assert_eq!(expand_tilde("/x/~y", home), PathBuf::from("/x/~y"));
        assert_eq!(expand_tilde("~user", home), PathBuf::from("~user"));
    }

    #[test]
    fn builds_locations_under_root() {
        let p = Paths::new("/r".into(), "/h".into());
        assert_eq!(p.config_file(), PathBuf::from("/r/config.toml"));
        assert_eq!(p.kind_dir(Kind::Agent), PathBuf::from("/r/library/agents"));
        assert_eq!(p.kind_dir(Kind::ClaudeMd), PathBuf::from("/r/library/claude-md"));
        assert_eq!(p.sessions_dir(), PathBuf::from("/r/sessions"));
        assert_eq!(p.run_dir(), PathBuf::from("/r/run"));
        assert_eq!(p.claude_dir(), PathBuf::from("/h/.claude"));
    }
}
