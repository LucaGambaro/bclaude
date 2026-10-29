use std::fs;

use crate::config::{Config, GLOBAL_PROFILE};
use crate::library::Library;
use crate::paths::Paths;
use crate::sessions::SessionRecord;
use crate::types::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Skills,
    Agents,
    ClaudeMd,
    Plugins,
    Mcp,
    Profiles,
    Sessions,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Skills,
        Tab::Agents,
        Tab::ClaudeMd,
        Tab::Plugins,
        Tab::Mcp,
        Tab::Profiles,
        Tab::Sessions,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Skills => "Skills",
            Tab::Agents => "Agents",
            Tab::ClaudeMd => "CLAUDE.md",
            Tab::Plugins => "Plugins",
            Tab::Mcp => "MCP",
            Tab::Profiles => "Profiles",
            Tab::Sessions => "Sessions",
        }
    }

    pub fn kind(self) -> Option<Kind> {
        match self {
            Tab::Skills => Some(Kind::Skill),
            Tab::Agents => Some(Kind::Agent),
            Tab::ClaudeMd => Some(Kind::ClaudeMd),
            Tab::Plugins => Some(Kind::Plugin),
            Tab::Mcp => Some(Kind::Mcp),
            _ => None,
        }
    }

    fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub detail: String,
    pub extra: String,
    /// Library tabs: is it part of the active profile.
    pub member: bool,
}

pub struct App {
    pub config: Config,
    pub library: Library,
    pub sessions: Vec<SessionRecord>,
    pub tab: Tab,
    pub selected: usize,
    pub active_profile: String,
    pub status: String,
    pub dirty: bool,
}

impl App {
    pub fn new(config: Config, library: Library, sessions: Vec<SessionRecord>) -> Self {
        Self {
            config,
            library,
            sessions,
            tab: Tab::Skills,
            selected: 0,
            active_profile: GLOBAL_PROFILE.to_string(),
            status: String::new(),
            dirty: false,
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        match self.tab.kind() {
            Some(kind) => self
                .library
                .of_kind(kind)
                .map(|e| Row {
                    name: e.name.clone(),
                    detail: e
                        .description
                        .clone()
                        .unwrap_or_else(|| "(no description)".into()),
                    extra: {
                        let p = self.config.profiles_including(kind, &e.name);
                        if p.is_empty() {
                            "-".into()
                        } else {
                            p.join(",")
                        }
                    },
                    member: self.config.is_member(&self.active_profile, kind, &e.name),
                })
                .collect(),
            None if self.tab == Tab::Profiles => self
                .config
                .profile_names()
                .into_iter()
                .map(|name| {
                    let p = self.config.profiles.get(&name).cloned().unwrap_or_default();
                    Row {
                        detail: if p.match_paths.is_empty() {
                            "(always / manual)".into()
                        } else {
                            p.match_paths.join(", ")
                        },
                        extra: format!(
                            "{}s {}a {}m {}p {}mcp",
                            p.skills.len(),
                            p.agents.len(),
                            p.claude_md.len(),
                            p.plugins.len(),
                            p.mcp.len()
                        ),
                        member: name == self.active_profile,
                        name,
                    }
                })
                .collect(),
            None => self
                .sessions
                .iter()
                .map(|s| Row {
                    name: s.cwd.clone(),
                    detail: s.profiles.join(","),
                    extra: format!("pid {}", s.pid),
                    member: false,
                })
                .collect(),
        }
    }

    pub fn selected_row(&self) -> Option<Row> {
        self.rows().into_iter().nth(self.selected)
    }

    pub fn next_tab(&mut self) {
        self.set_tab((self.tab.index() + 1) % Tab::ALL.len());
    }

    pub fn prev_tab(&mut self) {
        self.set_tab((self.tab.index() + Tab::ALL.len() - 1) % Tab::ALL.len());
    }

    fn set_tab(&mut self, index: usize) {
        self.tab = Tab::ALL[index];
        self.selected = 0;
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.rows().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        self.selected = self.selected.saturating_add_signed(delta).min(len - 1);
    }

    pub fn cycle_profile(&mut self) {
        let names = self.config.profile_names();
        let pos = names
            .iter()
            .position(|n| *n == self.active_profile)
            .unwrap_or(0);
        self.active_profile = names[(pos + 1) % names.len()].clone();
        self.status = format!("active profile: {}", self.active_profile);
    }

    /// Toggles the selected library entry in the active profile (library tabs only).
    pub fn toggle_membership(&mut self) {
        let (Some(kind), Some(row)) = (self.tab.kind(), self.selected_row()) else {
            self.status = "toggle works on Skills / Agents / CLAUDE.md / Plugins / MCP tabs".into();
            return;
        };
        let on = !row.member;
        if self
            .config
            .set_membership(&self.active_profile, kind, &row.name, on)
        {
            self.dirty = true;
            self.status = format!(
                "{} {} {} `{}`",
                if on { "added to" } else { "removed from" },
                self.active_profile,
                kind.label(),
                row.name
            );
        }
    }

    pub fn preview(&self) -> String {
        let Some(row) = self.selected_row() else {
            return String::new();
        };
        match self.tab.kind() {
            Some(Kind::Plugin) => self
                .library
                .find(Kind::Plugin, &row.name)
                .map(|e| format!("installed at:\n{}", e.path.display()))
                .unwrap_or_default(),
            // the file may hold secrets (env, headers): show only the summary
            Some(Kind::Mcp) => self
                .library
                .find(Kind::Mcp, &row.name)
                .and_then(|e| e.description.clone())
                .unwrap_or_default(),
            Some(kind) => self
                .library
                .find(kind, &row.name)
                .and_then(|e| fs::read_to_string(&e.path).ok())
                .map(|t| t.lines().take(40).collect::<Vec<_>>().join("\n"))
                .unwrap_or_else(|| "(cannot read file)".into()),
            None if self.tab == Tab::Profiles => {
                let p = self
                    .config
                    .profiles
                    .get(&row.name)
                    .cloned()
                    .unwrap_or_default();
                format!(
                    "skills: {}\nagents: {}\nclaude.md: {}\nplugins: {}\nmcp: {}\nmatch: {}",
                    p.skills.join(", "),
                    p.agents.join(", "),
                    p.claude_md.join(", "),
                    p.plugins.join(", "),
                    p.mcp.join(", "),
                    p.match_paths.join(", ")
                )
            }
            None => self
                .sessions
                .iter()
                .find(|s| s.cwd == row.name)
                .map(|s| {
                    format!(
                        "pid: {}\nskills: {}\nagents: {}\nclaude.md: {}\nplugins: {}\nmcp: {}",
                        s.pid,
                        s.skills.join(", "),
                        s.agents.join(", "),
                        s.claude_md.join(", "),
                        s.plugins.join(", "),
                        s.mcp.join(", ")
                    )
                })
                .unwrap_or_default(),
        }
    }

    /// Re-reads config, library and live sessions from disk.
    pub fn reload(&mut self, paths: &Paths) -> crate::error::Result<()> {
        self.config = crate::config::load(&paths.config_file())?;
        self.library = Library::load(paths)?;
        self.sessions = crate::sessions::list_alive(&paths.sessions_dir(), &paths.run_dir())?;
        self.move_selection(0);
        self.status = "reloaded".into();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Entry;
    use std::path::PathBuf;

    fn app() -> App {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("SKILL.md");
        fs::write(&file, "line1\nline2").unwrap();
        let entry = |kind, name: &str, desc: Option<&str>| Entry {
            kind,
            name: name.into(),
            description: desc.map(String::from),
            path: file.clone(),
            size_bytes: 1,
        };
        let library = Library::from_entries(vec![
            entry(Kind::Skill, "a", Some("first")),
            entry(Kind::Skill, "b", None),
            entry(Kind::Agent, "ag", None),
        ]);
        let mut config = Config::default();
        config.set_membership("global", Kind::Skill, "a", true);
        config.set_membership("px", Kind::Skill, "a", true);
        config
            .profiles
            .get_mut("px")
            .unwrap()
            .match_paths
            .push("~/px/*".into());
        let sessions = vec![SessionRecord {
            pid: 5,
            cwd: "/w".into(),
            profiles: vec!["global".into()],
            skills: vec!["a".into()],
            agents: vec![],
            claude_md: vec![],
            plugins: vec![],
            mcp: vec![],
            started_unix: 1,
        }];
        // keep the tempdir alive by leaking it: the file is read in preview tests
        std::mem::forget(dir);
        App::new(config, library, sessions)
    }

    #[test]
    fn library_rows_show_membership_and_profiles() {
        let a = app();
        let rows = a.rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            (
                rows[0].name.as_str(),
                rows[0].member,
                rows[0].extra.as_str()
            ),
            ("a", true, "global,px")
        );
        assert_eq!(
            (
                rows[1].member,
                rows[1].extra.as_str(),
                rows[1].detail.as_str()
            ),
            (false, "-", "(no description)")
        );
    }

    #[test]
    fn tabs_cycle_and_reset_selection() {
        let mut a = app();
        a.selected = 1;
        a.next_tab();
        assert_eq!((a.tab, a.selected), (Tab::Agents, 0));
        a.prev_tab();
        a.prev_tab();
        assert_eq!(a.tab, Tab::Sessions);
        a.next_tab();
        assert_eq!(a.tab, Tab::Skills);
        assert_eq!(Tab::ClaudeMd.title(), "CLAUDE.md");
    }

    #[test]
    fn selection_is_clamped() {
        let mut a = app();
        a.move_selection(-1);
        assert_eq!(a.selected, 0);
        a.move_selection(10);
        assert_eq!(a.selected, 1);
        a.tab = Tab::ClaudeMd;
        a.move_selection(1);
        assert_eq!(a.selected, 0);
        assert!(a.selected_row().is_none());
        assert_eq!(a.preview(), "");
    }

    #[test]
    fn toggle_adds_and_removes_in_active_profile() {
        let mut a = app();
        a.selected = 1;
        a.toggle_membership();
        assert!(a.config.is_member("global", Kind::Skill, "b") && a.dirty);
        assert!(a.status.contains("added to global"));
        a.toggle_membership();
        assert!(!a.config.is_member("global", Kind::Skill, "b"));
        assert!(a.status.contains("removed from global"));
    }

    #[test]
    fn toggle_outside_library_tabs_explains() {
        let mut a = app();
        a.tab = Tab::Profiles;
        a.toggle_membership();
        assert!(!a.dirty);
        assert!(a.status.contains("toggle works"));
    }

    #[test]
    fn cycle_profile_wraps() {
        let mut a = app();
        a.cycle_profile();
        assert_eq!(a.active_profile, "px");
        a.cycle_profile();
        assert_eq!(a.active_profile, "global");
        a.cycle_profile();
        a.toggle_membership();
        assert!(!a.config.is_member("px", Kind::Skill, "a"));
    }

    #[test]
    fn profiles_and_sessions_rows_and_previews() {
        let mut a = app();
        a.tab = Tab::Profiles;
        let rows = a.rows();
        assert_eq!(rows[1].name, "px");
        assert_eq!(rows[1].detail, "~/px/*");
        assert_eq!(rows[0].detail, "(always / manual)");
        assert_eq!(rows[1].extra, "1s 0a 0m 0p 0mcp");
        assert!(rows[0].member);
        a.selected = 1;
        assert!(a.preview().contains("match: ~/px/*"));
        a.tab = Tab::Sessions;
        a.selected = 0;
        assert_eq!(a.rows()[0].extra, "pid 5");
        assert!(a.preview().contains("skills: a"));
    }

    #[test]
    fn plugins_tab_toggles_membership_and_previews_install_path() {
        let mut a = app();
        a.library = Library::from_entries(vec![Entry {
            kind: Kind::Plugin,
            name: "p@mk".into(),
            description: Some("Plug".into()),
            path: PathBuf::from("/installed/p"),
            size_bytes: 0,
        }]);
        a.tab = Tab::Plugins;
        assert_eq!(a.rows()[0].detail, "Plug");
        assert_eq!(a.preview(), "installed at:\n/installed/p");
        a.toggle_membership();
        assert!(a.config.is_member("global", Kind::Plugin, "p@mk"));
        assert!(a.rows()[0].member);
        a.tab = Tab::Profiles;
        assert_eq!(a.rows()[0].extra, "1s 0a 0m 1p 0mcp");
        assert!(a.preview().contains("plugins: p@mk"));
    }

    #[test]
    fn mcp_preview_shows_only_the_summary() {
        let mut a = app();
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("srv.json");
        fs::write(&file, r#"{"command":"uvx","env":{"TOKEN":"secret"}}"#).unwrap();
        a.library = Library::from_entries(vec![Entry {
            kind: Kind::Mcp,
            name: "srv".into(),
            description: Some("uvx".into()),
            path: file,
            size_bytes: 1,
        }]);
        a.tab = Tab::Mcp;
        assert_eq!(a.preview(), "uvx");
        a.toggle_membership();
        assert!(a.config.is_member("global", Kind::Mcp, "srv"));
        a.tab = Tab::Profiles;
        assert_eq!(a.rows()[0].extra, "1s 0a 0m 0p 1mcp");
        assert!(a.preview().contains("mcp: srv"));
    }

    #[test]
    fn preview_reads_file_head() {
        let a = app();
        assert_eq!(a.preview(), "line1\nline2");
        let mut broken = app();
        broken.library = Library::from_entries(vec![Entry {
            kind: Kind::Skill,
            name: "a".into(),
            description: None,
            path: PathBuf::from("/nonexistent/file"),
            size_bytes: 0,
        }]);
        assert_eq!(broken.preview(), "(cannot read file)");
    }

    #[test]
    fn reload_picks_up_disk_changes() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().join("bc"), dir.path().join("h"));
        fs::create_dir_all(paths.config_file().parent().unwrap()).unwrap();
        fs::write(paths.config_file(), "[profiles.zz]\nskills=[\"q\"]\n").unwrap();
        let mut a = app();
        a.reload(&paths).unwrap();
        assert!(a.config.profiles.contains_key("zz"));
        assert!(a.library.entries().is_empty());
        assert!(a.sessions.is_empty());
        assert_eq!(a.status, "reloaded");
    }
}
