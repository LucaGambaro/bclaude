use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
pub enum Kind {
    Skill,
    Agent,
    #[value(name = "md")]
    ClaudeMd,
    Plugin,
    Mcp,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Skill,
        Kind::Agent,
        Kind::ClaudeMd,
        Kind::Plugin,
        Kind::Mcp,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Skill => "skill",
            Kind::Agent => "agent",
            Kind::ClaudeMd => "md",
            Kind::Plugin => "plugin",
            Kind::Mcp => "mcp",
        }
    }

    /// Directory name inside the library.
    pub fn subdir(self) -> &'static str {
        match self {
            Kind::Skill => "skills",
            Kind::Agent => "agents",
            Kind::ClaudeMd => "claude-md",
            Kind::Plugin => "plugins",
            Kind::Mcp => "mcp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_subdirs_are_distinct() {
        let labels: Vec<_> = Kind::ALL.iter().map(|k| k.label()).collect();
        let dirs: Vec<_> = Kind::ALL.iter().map(|k| k.subdir()).collect();
        assert_eq!(labels, ["skill", "agent", "md", "plugin", "mcp"]);
        assert_eq!(dirs, ["skills", "agents", "claude-md", "plugins", "mcp"]);
    }
}
