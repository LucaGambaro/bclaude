use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::Kind;

pub const GLOBAL_PROFILE: &str = "global";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    pub projects: BTreeMap<String, ProjectOverride>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude_md: Vec<String>,
    /// Glob patterns of repo paths this profile applies to.
    #[serde(default, rename = "match", skip_serializing_if = "Vec::is_empty")]
    pub match_paths: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectOverride {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude_md: Vec<String>,
}

impl Profile {
    pub fn list(&self, kind: Kind) -> &Vec<String> {
        match kind {
            Kind::Skill => &self.skills,
            Kind::Agent => &self.agents,
            Kind::ClaudeMd => &self.claude_md,
        }
    }

    pub fn list_mut(&mut self, kind: Kind) -> &mut Vec<String> {
        match kind {
            Kind::Skill => &mut self.skills,
            Kind::Agent => &mut self.agents,
            Kind::ClaudeMd => &mut self.claude_md,
        }
    }
}

impl ProjectOverride {
    pub fn list(&self, kind: Kind) -> &Vec<String> {
        match kind {
            Kind::Skill => &self.skills,
            Kind::Agent => &self.agents,
            Kind::ClaudeMd => &self.claude_md,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let cfg: Config = toml::from_str(
            r#"
[profiles.global]
skills = ["a"]
[profiles.px]
description = "acme"
agents = ["x"]
claude_md = ["m"]
match = ["~/w/*"]
[projects."~/w/foo"]
profiles = ["px"]
skills = ["extra"]
"#,
        )
        .unwrap();
        assert_eq!(cfg.profiles["global"].skills, ["a"]);
        assert_eq!(cfg.profiles["px"].match_paths, ["~/w/*"]);
        assert_eq!(cfg.profiles["px"].description.as_deref(), Some("acme"));
        assert_eq!(cfg.projects["~/w/foo"].list(Kind::Skill), &["extra"]);
        assert!(cfg.projects["~/w/foo"].list(Kind::Agent).is_empty());
    }

    #[test]
    fn empty_input_is_default() {
        assert_eq!(toml::from_str::<Config>("").unwrap(), Config::default());
    }

    #[test]
    fn roundtrips_through_toml() {
        let mut cfg = Config::default();
        let p = cfg.profiles.entry("p".into()).or_default();
        p.list_mut(Kind::ClaudeMd).push("m".into());
        p.match_paths.push("~/x/*".into());
        let text = toml::to_string_pretty(&cfg).unwrap();
        assert!(text.contains("match = "));
        assert_eq!(toml::from_str::<Config>(&text).unwrap(), cfg);
    }
}
