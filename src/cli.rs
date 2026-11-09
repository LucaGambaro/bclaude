use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands;
use crate::error::Result;
use crate::resolve::ProfileRequest;
use crate::types::Kind;

const BUILTINS: [&str; 6] = ["ls", "which", "add", "import", "dash", "help"];

#[derive(Debug, Parser)]
#[command(
    name = "bclaude",
    about = "betterClaude: compose skills, agents and CLAUDE.md per project on top of `claude`"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// List library entries with their profiles and descriptions
    Ls {
        #[arg(long, value_enum, ignore_case = true)]
        kind: Option<Kind>,
    },
    /// Show what applies to the current directory and why
    Which,
    /// Copy a skill dir / agent .md / CLAUDE.md fragment into the library
    Add {
        #[arg(value_enum, ignore_case = true)]
        kind: Kind,
        path: PathBuf,
        #[arg(long)]
        name: Option<String>,
    },
    /// Import skills and agents from ~/.claude into the library
    Import,
    /// Open the TUI dashboard
    Dash,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Route {
    Builtin(Vec<String>),
    /// Launch claude with the composed config; remaining args go to claude untouched.
    Run {
        dry_run: bool,
        /// `--profile` / `--exclusive`: profiles requested on the command line.
        request: ProfileRequest,
        args: Vec<String>,
    },
}

/// `bc <builtin> ...` is handled here; anything else is forwarded to claude.
/// Leading `--dry-run`, `--exclusive` and `--profile <name>` / `--profile=<name>` are consumed by bclaude.
pub fn route(args: Vec<String>) -> Route {
    if let Some(first) = args.first().map(String::as_str)
        && (BUILTINS.contains(&first) || first == "--help" || first == "-h")
    {
        return Route::Builtin(args);
    }
    let mut dry_run = false;
    let mut request = ProfileRequest::default();
    let mut rest = args.as_slice();
    loop {
        match rest {
            [flag, tail @ ..] if flag == "--dry-run" => {
                dry_run = true;
                rest = tail;
            }
            [flag, tail @ ..] if flag == "--exclusive" => {
                request.exclusive = true;
                rest = tail;
            }
            [flag, name, tail @ ..] if flag == "--profile" => {
                request.names.push(name.clone());
                rest = tail;
            }
            [flag, tail @ ..] if flag.starts_with("--profile=") => {
                request.names.push(flag["--profile=".len()..].to_string());
                rest = tail;
            }
            _ => break,
        }
    }
    Route::Run {
        dry_run,
        request,
        args: rest.to_vec(),
    }
}

pub fn run(args: Vec<String>) -> Result<i32> {
    match route(args) {
        Route::Run {
            dry_run,
            request,
            args,
        } => commands::run(dry_run, &request, &args),
        Route::Builtin(args) => {
            let argv = std::iter::once("bclaude".to_string()).chain(args);
            match Cli::try_parse_from(argv) {
                Ok(cli) => commands::dispatch(cli.command),
                Err(err) => {
                    let code = err.exit_code();
                    err.print().ok();
                    Ok(code)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn builtins_are_routed_to_clap() {
        assert_eq!(
            route(v(&["ls", "--kind", "skill"])),
            Route::Builtin(v(&["ls", "--kind", "skill"]))
        );
        assert_eq!(route(v(&["--help"])), Route::Builtin(v(&["--help"])));
        assert_eq!(route(v(&["dash"])), Route::Builtin(v(&["dash"])));
    }

    #[test]
    fn everything_else_goes_to_claude() {
        assert_eq!(
            route(v(&[])),
            Route::Run {
                dry_run: false,
                request: ProfileRequest::default(),
                args: vec![]
            }
        );
        assert_eq!(
            route(v(&["-c", "--model", "x"])),
            Route::Run {
                dry_run: false,
                request: ProfileRequest::default(),
                args: v(&["-c", "--model", "x"])
            }
        );
        assert_eq!(
            route(v(&["fix the bug"])),
            Route::Run {
                dry_run: false,
                request: ProfileRequest::default(),
                args: v(&["fix the bug"])
            }
        );
    }

    #[test]
    fn dry_run_flag_is_consumed() {
        assert_eq!(
            route(v(&["--dry-run", "-p", "hi"])),
            Route::Run {
                dry_run: true,
                request: ProfileRequest::default(),
                args: v(&["-p", "hi"])
            }
        );
    }

    #[test]
    fn profile_flags_are_consumed_in_any_order_before_claude_args() {
        assert_eq!(
            route(v(&["--profile", "a", "--dry-run", "--profile=b", "-c"])),
            Route::Run {
                dry_run: true,
                request: ProfileRequest {
                    names: v(&["a", "b"]),
                    exclusive: false
                },
                args: v(&["-c"])
            }
        );
    }

    #[test]
    fn exclusive_flag_is_consumed_with_profile() {
        assert_eq!(
            route(v(&["--exclusive", "--profile", "a", "-c"])),
            Route::Run {
                dry_run: false,
                request: ProfileRequest {
                    names: v(&["a"]),
                    exclusive: true
                },
                args: v(&["-c"])
            }
        );
    }

    #[test]
    fn profile_flag_after_claude_args_is_forwarded_untouched() {
        assert_eq!(
            route(v(&["-c", "--profile", "a"])),
            Route::Run {
                dry_run: false,
                request: ProfileRequest::default(),
                args: v(&["-c", "--profile", "a"])
            }
        );
        // a dangling `--profile` without a value is left for claude to report
        assert_eq!(
            route(v(&["--profile"])),
            Route::Run {
                dry_run: false,
                request: ProfileRequest::default(),
                args: v(&["--profile"])
            }
        );
    }

    #[test]
    fn clap_parses_subcommands() {
        let ls = Cli::try_parse_from(["bclaude", "ls", "--kind", "md"]).unwrap();
        assert_eq!(
            ls.command,
            Command::Ls {
                kind: Some(Kind::ClaudeMd)
            }
        );
        let add = Cli::try_parse_from(["bclaude", "add", "skill", "/p", "--name", "n"]).unwrap();
        assert_eq!(
            add.command,
            Command::Add {
                kind: Kind::Skill,
                path: "/p".into(),
                name: Some("n".into())
            }
        );
        assert_eq!(
            Cli::try_parse_from(["bclaude", "which"]).unwrap().command,
            Command::Which
        );
        assert!(Cli::try_parse_from(["bclaude", "add", "bogus", "/p"]).is_err());
    }

    #[test]
    fn kind_values_ignore_case() {
        let ls = Cli::try_parse_from(["bclaude", "ls", "--kind", "MCP"]).unwrap();
        assert_eq!(
            ls.command,
            Command::Ls {
                kind: Some(Kind::Mcp)
            }
        );
        let add = Cli::try_parse_from(["bclaude", "add", "Skill", "/p"]).unwrap();
        assert!(matches!(
            add.command,
            Command::Add {
                kind: Kind::Skill,
                ..
            }
        ));
    }
}
