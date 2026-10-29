use std::collections::BTreeMap;

use serde::Deserialize;

/// `~/.claude/plugins/installed_plugins.json`; each id maps to one install per scope.
#[derive(Debug, Default, Deserialize)]
pub struct InstalledPlugins {
    #[serde(default)]
    pub plugins: BTreeMap<String, Vec<Install>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    pub install_path: String,
}

/// `<installPath>/.claude-plugin/plugin.json` (only what is shown to the user).
#[derive(Debug, Default, Deserialize)]
pub struct PluginManifest {
    #[serde(default)]
    pub description: Option<String>,
}
