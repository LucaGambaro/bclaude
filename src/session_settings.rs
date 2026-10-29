use std::collections::BTreeMap;

use serde::Serialize;

/// The per-session `--settings` file: only what betterClaude needs to override.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettings {
    pub enabled_plugins: BTreeMap<String, bool>,
}

impl SessionSettings {
    /// Enables `selected` and disables every other installed plugin, so only the chosen ones load.
    pub fn for_plugins<'a>(
        selected: &[String],
        installed: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let mut enabled_plugins: BTreeMap<String, bool> = installed
            .into_iter()
            .map(|id| (id.to_string(), false))
            .collect();
        for id in selected {
            enabled_plugins.insert(id.clone(), true);
        }
        Self { enabled_plugins }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enables_selected_and_disables_the_rest() {
        let s = SessionSettings::for_plugins(&["a@m".into()], ["b@m", "a@m", "c@m"]);
        assert_eq!(
            s.enabled_plugins.into_iter().collect::<Vec<_>>(),
            [
                ("a@m".to_string(), true),
                ("b@m".to_string(), false),
                ("c@m".to_string(), false)
            ]
        );
    }

    #[test]
    fn serializes_with_camel_case_key() {
        let s = SessionSettings::for_plugins(&["a@m".into()], ["b@m"]);
        assert_eq!(
            serde_json::to_string(&s).unwrap(),
            r#"{"enabledPlugins":{"a@m":true,"b@m":false}}"#
        );
    }
}
