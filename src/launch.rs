use std::ffi::OsString;
use std::process::Command;

use crate::compose::Composed;
use crate::error::{BcError, Result};

pub const DEFAULT_BIN: &str = "claude";

pub fn claude_bin() -> String {
    std::env::var("BC_CLAUDE_BIN").unwrap_or_else(|_| DEFAULT_BIN.to_string())
}

/// Composed flags first, then the user's own arguments untouched.
pub fn build_args(composed: &Composed, user_args: &[String]) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    if let Some(dir) = &composed.plugin_dir {
        args.push("--plugin-dir".into());
        args.push(dir.clone().into_os_string());
    }
    if let Some(file) = &composed.settings_file {
        args.push("--settings".into());
        args.push(file.clone().into_os_string());
    }
    if let Some(file) = &composed.mcp_config {
        args.push("--mcp-config".into());
        args.push(file.clone().into_os_string());
        args.push("--strict-mcp-config".into());
    }
    if let Some(file) = &composed.claude_md {
        args.push("--append-system-prompt-file".into());
        args.push(file.clone().into_os_string());
    }
    args.extend(user_args.iter().map(OsString::from));
    args
}

pub fn render_command(bin: &str, args: &[OsString]) -> String {
    let mut parts = vec![bin.to_string()];
    parts.extend(args.iter().map(|a| {
        let s = a.to_string_lossy();
        if s.is_empty() || s.contains(char::is_whitespace) {
            format!("'{s}'")
        } else {
            s.into_owned()
        }
    }));
    parts.join(" ")
}

/// Runs claude attached to the current terminal and returns its exit code.
pub fn run(bin: &str, args: &[OsString]) -> Result<i32> {
    let status = Command::new(bin)
        .args(args)
        .status()
        .map_err(|source| BcError::Launch {
            bin: bin.to_string(),
            source,
        })?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn composed(plugin: Option<&str>, md: Option<&str>) -> Composed {
        Composed {
            plugin_dir: plugin.map(PathBuf::from),
            claude_md: md.map(PathBuf::from),
            settings_file: None,
            mcp_config: None,
            missing: vec![],
        }
    }

    fn strs(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn builds_flags_then_user_args() {
        let args = build_args(
            &composed(Some("/p"), Some("/m")),
            &["-c".into(), "--model".into(), "x".into()],
        );
        assert_eq!(
            strs(&args),
            [
                "--plugin-dir",
                "/p",
                "--append-system-prompt-file",
                "/m",
                "-c",
                "--model",
                "x"
            ]
        );
    }

    #[test]
    fn passes_settings_between_plugin_dir_and_claude_md() {
        let mut c = composed(Some("/p"), Some("/m"));
        c.settings_file = Some(PathBuf::from("/s.json"));
        assert_eq!(
            strs(&build_args(&c, &[])),
            [
                "--plugin-dir",
                "/p",
                "--settings",
                "/s.json",
                "--append-system-prompt-file",
                "/m"
            ]
        );
    }

    #[test]
    fn mcp_config_is_strict() {
        let mut c = composed(None, None);
        c.mcp_config = Some(PathBuf::from("/m.json"));
        assert_eq!(
            strs(&build_args(&c, &["-c".into()])),
            ["--mcp-config", "/m.json", "--strict-mcp-config", "-c"]
        );
    }

    #[test]
    fn omits_flags_when_nothing_composed() {
        assert_eq!(
            strs(&build_args(&composed(None, None), &["-p".into()])),
            ["-p"]
        );
        assert_eq!(
            strs(&build_args(&composed(None, Some("/m")), &[])),
            ["--append-system-prompt-file", "/m"]
        );
    }

    #[test]
    fn renders_quoting_whitespace_args() {
        let args = vec![
            OsString::from("--x"),
            OsString::from("a b"),
            OsString::from(""),
        ];
        assert_eq!(render_command("claude", &args), "claude --x 'a b' ''");
    }

    #[test]
    fn run_returns_exit_code() {
        assert_eq!(run("true", &[]).unwrap(), 0);
        assert_eq!(run("sh", &["-c".into(), "exit 3".into()]).unwrap(), 3);
    }

    #[test]
    fn run_reports_missing_binary() {
        assert!(matches!(
            run("definitely-not-a-binary-bc", &[]),
            Err(BcError::Launch { .. })
        ));
    }
}
