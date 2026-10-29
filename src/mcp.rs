use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{BcError, Result};

/// One MCP server definition (stdio, http, sse...), passed to claude untouched.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerDef(Map<String, Value>);

/// The `--mcp-config` file generated for a session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    pub mcp_servers: BTreeMap<String, McpServerDef>,
}

/// The only part of `~/.claude.json` betterClaude cares about.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeJson {
    #[serde(default)]
    mcp_servers: BTreeMap<String, McpServerDef>,
}

impl McpServerDef {
    pub fn parse(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text)
    }

    pub fn read(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|e| BcError::io(path, e))?;
        Self::parse(&text).map_err(|source| BcError::McpJson {
            path: path.to_path_buf(),
            source,
        })
    }

    /// One-line summary for listings: the url, or the command with its args.
    /// Never includes `env`/`headers`, which may hold secrets.
    pub fn summary(&self) -> Option<String> {
        if let Some(url) = self.0.get("url").and_then(Value::as_str) {
            return Some(url.to_string());
        }
        let command = self.0.get("command").and_then(Value::as_str)?;
        let args = self
            .0
            .get("args")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str);
        Some(
            std::iter::once(command)
                .chain(args)
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

/// User-scope servers registered in Claude Code's own `~/.claude.json`.
pub fn registered_servers(path: &Path) -> Result<BTreeMap<String, McpServerDef>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(BcError::io(path, e)),
    };
    let parsed: ClaudeJson = serde_json::from_str(&text).map_err(|source| BcError::McpJson {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(parsed.mcp_servers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(text: &str) -> McpServerDef {
        McpServerDef::parse(text).unwrap()
    }

    #[test]
    fn summary_prefers_url_then_command_with_args() {
        assert_eq!(
            def(r#"{"type":"http","url":"https://x/mcp"}"#)
                .summary()
                .as_deref(),
            Some("https://x/mcp")
        );
        assert_eq!(
            def(r#"{"command":"uvx","args":["serena","--n",1],"env":{"K":"secret"}}"#)
                .summary()
                .as_deref(),
            Some("uvx serena --n")
        );
        assert_eq!(
            def(r#"{"command":"solo"}"#).summary().as_deref(),
            Some("solo")
        );
        assert_eq!(def("{}").summary(), None);
    }

    #[test]
    fn rejects_non_object_definitions() {
        assert!(McpServerDef::parse("[1]").is_err());
        assert!(McpServerDef::parse("not json").is_err());
    }

    #[test]
    fn read_reports_io_and_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            McpServerDef::read(&dir.path().join("none.json")),
            Err(BcError::Io { .. })
        ));
        let bad = dir.path().join("bad.json");
        fs::write(&bad, "[]").unwrap();
        assert!(matches!(
            McpServerDef::read(&bad),
            Err(BcError::McpJson { .. })
        ));
        let good = dir.path().join("good.json");
        fs::write(&good, r#"{"command":"c"}"#).unwrap();
        assert_eq!(
            McpServerDef::read(&good).unwrap().summary().as_deref(),
            Some("c")
        );
    }

    #[test]
    fn config_serializes_under_mcp_servers() {
        let mut cfg = McpConfig::default();
        cfg.mcp_servers
            .insert("s".into(), def(r#"{"command":"c"}"#));
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            r#"{"mcpServers":{"s":{"command":"c"}}}"#
        );
    }

    #[test]
    fn registered_servers_reads_only_mcp_servers() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            registered_servers(&dir.path().join("none"))
                .unwrap()
                .is_empty()
        );
        let file = dir.path().join(".claude.json");
        fs::write(
            &file,
            r#"{"numStartups":3,"projects":{},"mcpServers":{"serena":{"command":"uvx"}}}"#,
        )
        .unwrap();
        let servers = registered_servers(&file).unwrap();
        assert_eq!(servers.keys().collect::<Vec<_>>(), ["serena"]);
        fs::write(&file, "{").unwrap();
        assert!(matches!(
            registered_servers(&file),
            Err(BcError::McpJson { .. })
        ));
        assert!(matches!(
            registered_servers(dir.path()),
            Err(BcError::Io { .. })
        ));
    }
}
