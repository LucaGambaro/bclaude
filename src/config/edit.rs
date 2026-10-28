use super::schema::{Config, GLOBAL_PROFILE};
use crate::types::Kind;

impl Config {
    /// Adds or removes `name` from the profile's list (creating the profile if needed).
    /// Returns true when the config changed.
    pub fn set_membership(&mut self, profile: &str, kind: Kind, name: &str, on: bool) -> bool {
        let list = self
            .profiles
            .entry(profile.to_string())
            .or_default()
            .list_mut(kind);
        let present = list.iter().any(|n| n == name);
        match (on, present) {
            (true, false) => {
                list.push(name.to_string());
                true
            }
            (false, true) => {
                list.retain(|n| n != name);
                true
            }
            _ => false,
        }
    }

    pub fn is_member(&self, profile: &str, kind: Kind, name: &str) -> bool {
        self.profiles
            .get(profile)
            .is_some_and(|p| p.list(kind).iter().any(|n| n == name))
    }

    /// Profiles (sorted) that include the entry.
    pub fn profiles_including(&self, kind: Kind, name: &str) -> Vec<String> {
        self.profiles
            .iter()
            .filter(|(_, p)| p.list(kind).iter().any(|n| n == name))
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Profile names, always including `global`.
    pub fn profile_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.profiles.keys().cloned().collect();
        if !names.iter().any(|n| n == GLOBAL_PROFILE) {
            names.insert(0, GLOBAL_PROFILE.to_string());
        }
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_is_idempotent() {
        let mut c = Config::default();
        assert!(c.set_membership("g", Kind::Skill, "a", true));
        assert!(!c.set_membership("g", Kind::Skill, "a", true));
        assert!(c.is_member("g", Kind::Skill, "a"));
        assert!(!c.is_member("g", Kind::Agent, "a"));
        assert!(c.set_membership("g", Kind::Skill, "a", false));
        assert!(!c.set_membership("g", Kind::Skill, "a", false));
        assert!(!c.is_member("g", Kind::Skill, "a"));
    }

    #[test]
    fn lists_profiles_including_entry() {
        let mut c = Config::default();
        c.set_membership("b", Kind::Agent, "x", true);
        c.set_membership("a", Kind::Agent, "x", true);
        c.set_membership("c", Kind::Agent, "y", true);
        assert_eq!(c.profiles_including(Kind::Agent, "x"), ["a", "b"]);
        assert!(c.profiles_including(Kind::Skill, "x").is_empty());
    }

    #[test]
    fn profile_names_always_has_global_first() {
        let mut c = Config::default();
        assert_eq!(c.profile_names(), ["global"]);
        c.set_membership("zz", Kind::Skill, "s", true);
        assert_eq!(c.profile_names(), ["global", "zz"]);
        c.set_membership("global", Kind::Skill, "s", true);
        assert_eq!(c.profile_names(), ["global", "zz"]);
    }
}
