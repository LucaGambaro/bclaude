use std::path::Path;

use crate::compose::Composed;
use crate::config::Config;
use crate::library::Library;
use crate::resolve::Resolved;
use crate::types::Kind;

fn pad(s: &str, width: usize) -> String {
    format!("{s:<width$}")
}

fn one_line(text: &str, max: usize) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() > max {
        let cut: String = flat.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    } else {
        flat
    }
}

/// Table of library entries with the profiles that include each (optionally one kind).
pub fn render_ls(library: &Library, config: &Config, kind: Option<Kind>) -> String {
    let entries: Vec<_> = library
        .entries()
        .iter()
        .filter(|e| kind.is_none_or(|k| e.kind == k))
        .collect();
    if entries.is_empty() {
        return "(library is empty — use `bclaude add` or `bclaude import`)\n".to_string();
    }
    let name_w = entries
        .iter()
        .map(|e| e.name.chars().count())
        .max()
        .unwrap_or(4)
        .max(4);
    let mut out = format!(
        "{} {} {} DESCRIPTION\n",
        pad("KIND", 5),
        pad("NAME", name_w),
        pad("PROFILES", 18)
    );
    for e in entries {
        let profiles = config.profiles_including(e.kind, &e.name);
        let profiles = if profiles.is_empty() {
            "-".to_string()
        } else {
            profiles.join(",")
        };
        out.push_str(&format!(
            "{} {} {} {}\n",
            pad(e.kind.label(), 5),
            pad(&e.name, name_w),
            pad(&one_line(&profiles, 18), 18),
            one_line(e.description.as_deref().unwrap_or("(no description)"), 80),
        ));
    }
    out
}

/// Explains what applies to `cwd`, why, and the estimated context cost.
pub fn render_which(
    cwd: &Path,
    resolved: &Resolved,
    composed: &Composed,
    library: &Library,
) -> String {
    let mut out = format!("cwd: {}\n\nprofiles:\n", cwd.display());
    if resolved.activations.is_empty() {
        out.push_str("  (none)\n");
    }
    for a in &resolved.activations {
        out.push_str(&format!("  {} — {}\n", a.profile, a.reason));
    }
    let mut total = 0;
    for kind in Kind::ALL {
        out.push_str(&format!("\n{}:\n", kind.label()));
        let names = resolved.names(kind);
        if names.is_empty() {
            out.push_str("  (none)\n");
        }
        for name in names {
            match library.find(kind, name) {
                Some(e) => {
                    total += e.estimated_tokens();
                    out.push_str(&format!("  {name} (~{} tok)\n", e.estimated_tokens()));
                }
                None => out.push_str(&format!("  {name} (MISSING in library)\n")),
            }
        }
    }
    out.push_str(&format!(
        "\nestimated context: ~{total} tokens (files only)\n"
    ));
    if !composed.missing.is_empty() {
        out.push_str(&format!(
            "warning: {} entries missing from the library\n",
            composed.missing.len()
        ));
    }
    out
}

/// Names-only version of `Composed` for dry-run (no filesystem needed).
pub fn missing_entries(resolved: &Resolved, library: &Library) -> Vec<(Kind, String)> {
    Kind::ALL
        .iter()
        .flat_map(|&k| resolved.names(k).iter().map(move |n| (k, n.clone())))
        .filter(|(k, n)| library.find(*k, n).is_none())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Entry;
    use std::path::PathBuf;

    fn entry(kind: Kind, name: &str, desc: Option<&str>, size: u64) -> Entry {
        Entry {
            kind,
            name: name.into(),
            description: desc.map(String::from),
            path: PathBuf::from("/x"),
            size_bytes: size,
        }
    }

    fn library() -> Library {
        Library::from_entries(vec![
            entry(Kind::Skill, "handoff", Some("Hand\noff   notes"), 40),
            entry(Kind::Agent, "rev", None, 8),
        ])
    }

    #[test]
    fn ls_shows_profiles_descriptions_and_filters_by_kind() {
        let mut cfg = Config::default();
        cfg.set_membership("global", Kind::Skill, "handoff", true);
        cfg.set_membership("px", Kind::Skill, "handoff", true);
        let all = render_ls(&library(), &cfg, None);
        assert!(all.contains("handoff"));
        assert!(all.contains("global,px"));
        assert!(all.contains("Hand off notes"));
        assert!(all.contains("(no description)"));
        let agents = render_ls(&library(), &cfg, Some(Kind::Agent));
        assert!(agents.contains("rev") && !agents.contains("handoff"));
    }

    #[test]
    fn ls_empty_library_hints_next_step() {
        assert!(
            render_ls(&Library::default(), &Config::default(), None).contains("bclaude import")
        );
    }

    #[test]
    fn long_descriptions_are_truncated() {
        let long = "x".repeat(200);
        let lib = Library::from_entries(vec![entry(Kind::Skill, "s", Some(&long), 1)]);
        let out = render_ls(&lib, &Config::default(), None);
        assert!(out.contains('…'));
        assert!(!out.contains(&long));
    }

    #[test]
    fn which_lists_profiles_tokens_and_missing() {
        let resolved = Resolved {
            activations: vec![crate::resolve::Activation {
                profile: "global".into(),
                reason: "always".into(),
            }],
            skills: vec!["handoff".into(), "ghost".into()],
            agents: vec![],
            claude_md: vec![],
        };
        let composed = Composed {
            plugin_dir: None,
            claude_md: None,
            missing: vec![(Kind::Skill, "ghost".into())],
        };
        let out = render_which(Path::new("/w"), &resolved, &composed, &library());
        assert!(out.contains("global — always"));
        assert!(out.contains("handoff (~10 tok)"));
        assert!(out.contains("ghost (MISSING in library)"));
        assert!(out.contains("~10 tokens"));
        assert!(out.contains("warning: 1 entries missing"));
        assert!(out.contains("agent:\n  (none)"));
    }

    #[test]
    fn which_without_profiles_says_none() {
        let composed = Composed {
            plugin_dir: None,
            claude_md: None,
            missing: vec![],
        };
        let out = render_which(
            Path::new("/w"),
            &Resolved::default(),
            &composed,
            &Library::default(),
        );
        assert!(out.contains("profiles:\n  (none)"));
        assert!(!out.contains("warning"));
    }

    #[test]
    fn missing_entries_finds_unknown_names() {
        let resolved = Resolved {
            skills: vec!["handoff".into(), "ghost".into()],
            agents: vec!["rev".into()],
            ..Default::default()
        };
        assert_eq!(
            missing_entries(&resolved, &library()),
            [(Kind::Skill, "ghost".to_string())]
        );
    }
}
