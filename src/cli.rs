use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands;
use crate::error::Result;
use crate::types::Kind;

const BUILTINS: [&str; 6] = ["ls", "which", "add", "import", "dash", "help"];

#[derive(Debug, Parser)]
#[command(name = "bclaude", about = "betterClaude: compose skills, agents and CLAUDE.md per project on top of `claude`")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// List library entries with their profiles and descriptions
    Ls {
        #[arg(long, value_enum)]
        kind: Option<Kind>,
    },
    /// Show what applies to the current directory and why
    Which,
    /// Copy a skill dir / agent .md / CLAUDE.md fragment into the library
    Add {
        #[arg(value_enum)]
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
    Run { dry_run: bool, args: Vec<String> },
}

/// `bc <builtin> ...` is handled here; anything else is forwarded to claude.
pub fn route(args: Vec<String>) -> Route {
    match args.first().map(String::as_str) {
        Some(first) if BUILTINS.contains(&first) || first == "--help" || first == "-h" => Route::Builtin(args),
        Some("--dry-run") => Route::Run { dry_run: true, args: args[1..].to_vec() },
        _ => Route::Run { dry_run: false, args },
    }
}

pub fn run(args: Vec<String>) -> Result<i32> {
    match route(args) {
        Route::Run { dry_run, args } => commands::run(dry_run, &args),
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
        assert_eq!(route(v(&["ls", "--kind", "skill"])), Route::Builtin(v(&["ls", "--kind", "skill"])));
        assert_eq!(route(v(&["--help"])), Route::Builtin(v(&["--help"])));
        assert_eq!(route(v(&["dash"])), Route::Builtin(v(&["dash"])));
    }

    #[test]
    fn everything_else_goes_to_claude() {
        assert_eq!(route(v(&[])), Route::Run { dry_run: false, args: vec![] });
        assert_eq!(route(v(&["-c", "--model", "x"])), Route::Run { dry_run: false, args: v(&["-c", "--model", "x"]) });
        assert_eq!(route(v(&["fix the bug"])), Route::Run { dry_run: false, args: v(&["fix the bug"]) });
    }

    #[test]
    fn dry_run_flag_is_consumed() {
        assert_eq!(route(v(&["--dry-run", "-p", "hi"])), Route::Run { dry_run: true, args: v(&["-p", "hi"]) });
    }

    #[test]
    fn clap_parses_subcommands() {
        let ls = Cli::try_parse_from(["bclaude", "ls", "--kind", "md"]).unwrap();
        assert_eq!(ls.command, Command::Ls { kind: Some(Kind::ClaudeMd) });
        let add = Cli::try_parse_from(["bclaude", "add", "skill", "/p", "--name", "n"]).unwrap();
        assert_eq!(add.command, Command::Add { kind: Kind::Skill, path: "/p".into(), name: Some("n".into()) });
        assert_eq!(Cli::try_parse_from(["bclaude", "which"]).unwrap().command, Command::Which);
        assert!(Cli::try_parse_from(["bclaude", "add", "bogus", "/p"]).is_err());
    }
}
