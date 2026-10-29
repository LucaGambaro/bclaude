use std::path::Path;

use glob::Pattern;

use crate::config::{Config, GLOBAL_PROFILE};
use crate::paths::expand_tilde;
use crate::types::Kind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activation {
    pub profile: String,
    pub reason: String,
}

/// Names of everything that applies to a working directory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resolved {
    pub activations: Vec<Activation>,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub claude_md: Vec<String>,
    pub plugins: Vec<String>,
    pub mcp: Vec<String>,
}

impl Resolved {
    pub fn names(&self, kind: Kind) -> &[String] {
        match kind {
            Kind::Skill => &self.skills,
            Kind::Agent => &self.agents,
            Kind::ClaudeMd => &self.claude_md,
            Kind::Plugin => &self.plugins,
            Kind::Mcp => &self.mcp,
        }
    }

    fn names_mut(&mut self, kind: Kind) -> &mut Vec<String> {
        match kind {
            Kind::Skill => &mut self.skills,
            Kind::Agent => &mut self.agents,
            Kind::ClaudeMd => &mut self.claude_md,
            Kind::Plugin => &mut self.plugins,
            Kind::Mcp => &mut self.mcp,
        }
    }

    fn extend_unique(&mut self, kind: Kind, names: &[String]) {
        let target = self.names_mut(kind);
        for name in names {
            if !target.contains(name) {
                target.push(name.clone());
            }
        }
    }

    pub fn profile_names(&self) -> Vec<String> {
        self.activations.iter().map(|a| a.profile.clone()).collect()
    }
}

/// True when the glob matches `cwd` or any of its ancestors (so subdirs of a repo match too).
fn pattern_matches(pattern: &str, cwd: &Path, home: &Path) -> bool {
    let expanded = expand_tilde(pattern, home);
    let Ok(glob) = Pattern::new(&expanded.to_string_lossy()) else {
        return false;
    };
    cwd.ancestors().any(|dir| glob.matches_path(dir))
}

/// Pure resolution: `global` (always), profiles whose `match` globs fit, then project overrides.
pub fn resolve(config: &Config, cwd: &Path, home: &Path) -> Resolved {
    let mut resolved = Resolved::default();
    let activate = |resolved: &mut Resolved, profile: &str, reason: String| {
        if !resolved.activations.iter().any(|a| a.profile == profile) {
            resolved.activations.push(Activation {
                profile: profile.to_string(),
                reason,
            });
        }
    };

    if config.profiles.contains_key(GLOBAL_PROFILE) {
        activate(&mut resolved, GLOBAL_PROFILE, "always".into());
    }
    for (name, profile) in &config.profiles {
        if name == GLOBAL_PROFILE {
            continue;
        }
        if let Some(pat) = profile
            .match_paths
            .iter()
            .find(|p| pattern_matches(p, cwd, home))
        {
            activate(&mut resolved, name, format!("match {pat}"));
        }
    }
    let overrides: Vec<_> = config
        .projects
        .iter()
        .filter(|(key, _)| cwd.starts_with(expand_tilde(key, home)))
        .collect();
    for (key, ov) in &overrides {
        for profile in &ov.profiles {
            activate(&mut resolved, profile, format!("project {key}"));
        }
    }

    for activation in resolved.activations.clone() {
        if let Some(profile) = config.profiles.get(&activation.profile) {
            for kind in Kind::ALL {
                resolved.extend_unique(kind, profile.list(kind));
            }
        }
    }
    for (_, ov) in overrides {
        for kind in Kind::ALL {
            resolved.extend_unique(kind, ov.list(kind));
        }
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn config() -> Config {
        toml::from_str(
            r#"
[profiles.global]
skills = ["g1"]
claude_md = ["personal"]
plugins = ["gp@mk"]
mcp = ["gm"]
[profiles.px]
skills = ["p1", "g1"]
agents = ["rev"]
match = ["~/work/px/*"]
[profiles.other]
skills = ["o1"]
match = ["~/work/other/*", "/bad[pattern"]
[projects."~/work/solo"]
profiles = ["other"]
skills = ["only-solo"]
"#,
        )
        .unwrap()
    }

    fn home() -> PathBuf {
        PathBuf::from("/h")
    }

    #[test]
    fn global_only_outside_any_group() {
        let r = resolve(&config(), Path::new("/tmp/x"), &home());
        assert_eq!(r.profile_names(), ["global"]);
        assert_eq!(r.skills, ["g1"]);
        assert_eq!(r.claude_md, ["personal"]);
        assert_eq!(r.plugins, ["gp@mk"]);
        assert_eq!(r.mcp, ["gm"]);
        assert!(r.agents.is_empty());
    }

    #[test]
    fn group_match_adds_profile_with_dedupe_and_order() {
        let r = resolve(&config(), Path::new("/h/work/px/repo"), &home());
        assert_eq!(r.profile_names(), ["global", "px"]);
        assert_eq!(r.skills, ["g1", "p1"]);
        assert_eq!(r.agents, ["rev"]);
        assert_eq!(r.activations[1].reason, "match ~/work/px/*");
    }

    #[test]
    fn subdirectory_of_repo_still_matches() {
        let r = resolve(&config(), Path::new("/h/work/px/repo/src/deep"), &home());
        assert!(r.profile_names().contains(&"px".to_string()));
    }

    #[test]
    fn sibling_group_does_not_match() {
        let r = resolve(&config(), Path::new("/h/work/other/r"), &home());
        assert_eq!(r.profile_names(), ["global", "other"]);
        assert_eq!(r.skills, ["g1", "o1"]);
    }

    #[test]
    fn project_override_adds_profiles_and_extras() {
        let r = resolve(&config(), Path::new("/h/work/solo/sub"), &home());
        assert_eq!(r.profile_names(), ["global", "other"]);
        assert_eq!(r.activations[1].reason, "project ~/work/solo");
        assert_eq!(r.skills, ["g1", "o1", "only-solo"]);
    }

    #[test]
    fn project_override_does_not_duplicate_matching_profile() {
        let mut cfg = config();
        cfg.projects.insert(
            "~/work/px/repo".into(),
            crate::config::ProjectOverride {
                profiles: vec!["px".into()],
                ..Default::default()
            },
        );
        let r = resolve(&cfg, Path::new("/h/work/px/repo"), &home());
        assert_eq!(r.profile_names(), ["global", "px"]);
        assert_eq!(r.activations[1].reason, "match ~/work/px/*");
    }

    #[test]
    fn empty_config_resolves_to_nothing_and_unknown_profile_is_ignored() {
        assert_eq!(
            resolve(&Config::default(), Path::new("/x"), &home()),
            Resolved::default()
        );
        let mut cfg = Config::default();
        cfg.projects.insert(
            "/x".into(),
            crate::config::ProjectOverride {
                profiles: vec!["ghost".into()],
                ..Default::default()
            },
        );
        let r = resolve(&cfg, Path::new("/x"), &home());
        assert_eq!(r.profile_names(), ["ghost"]);
        assert!(r.skills.is_empty());
    }
}
