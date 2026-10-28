use std::fs;
use std::io::Write;
use std::path::Path;

use crate::cli::Command;
use crate::compose::Composed;
use crate::config;
use crate::error::{BcError, Result};
use crate::launch;
use crate::library::{self, Library};
use crate::paths::Paths;
use crate::report;
use crate::resolve::resolve;
use crate::runner;
use crate::tui;

fn cwd() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(|e| BcError::io(".", e))
}

fn emit(out: &mut impl Write, text: &str) -> Result<()> {
    out.write_all(text.as_bytes()).map_err(|e| BcError::io("stdout", e))
}

/// Default action: run `claude` with the composed config (or just print the command).
pub fn run(dry_run: bool, args: &[String]) -> Result<i32> {
    let paths = Paths::from_env()?;
    let cwd = cwd()?;
    if dry_run {
        return dry_run_report(&paths, &cwd, args, &mut std::io::stdout(), &mut std::io::stderr());
    }
    runner::run_session(&paths, &cwd, args, &launch::claude_bin())
}

fn dry_run_report(
    paths: &Paths,
    cwd: &Path,
    args: &[String],
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<i32> {
    let prepared = runner::prepare(paths, cwd, std::process::id(), args)?;
    emit(out, &format!("{}\n", launch::render_command(&launch::claude_bin(), &prepared.args)))?;
    for (kind, name) in &prepared.composed.missing {
        emit(err, &format!("bclaude: warning: {} `{name}` not found in library\n", kind.label()))?;
    }
    // the dry run only previews: don't leave the generated session behind
    fs::remove_dir_all(&prepared.session_dir).map_err(|e| BcError::io(&prepared.session_dir, e))?;
    Ok(0)
}

pub fn dispatch(command: Command) -> Result<i32> {
    let paths = Paths::from_env()?;
    if command == Command::Dash {
        tui::run(&paths)?;
        return Ok(0);
    }
    execute(command, &paths, &cwd()?, &mut std::io::stdout())
}

/// Non-interactive commands, with all environment injected so they can be tested.
pub fn execute(command: Command, paths: &Paths, cwd: &Path, out: &mut impl Write) -> Result<i32> {
    match command {
        Command::Ls { kind } => {
            let cfg = config::load(&paths.config_file())?;
            let lib = Library::scan(&paths.library_dir())?;
            emit(out, &report::render_ls(&lib, &cfg, kind))?;
        }
        Command::Which => {
            let cfg = config::load(&paths.config_file())?;
            let lib = Library::scan(&paths.library_dir())?;
            let resolved = resolve(&cfg, cwd, &paths.home);
            let composed = Composed {
                plugin_dir: None,
                claude_md: None,
                missing: report::missing_entries(&resolved, &lib),
            };
            emit(out, &report::render_which(cwd, &resolved, &composed, &lib))?;
        }
        Command::Add { kind, path, name } => {
            let added = library::import::add(paths, kind, &path, name.as_deref())?;
            emit(out, &format!("added {} `{added}` to the library\n", kind.label()))?;
        }
        Command::Import => {
            let imported = library::import::import_from_claude(paths)?;
            if imported.is_empty() {
                emit(out, "nothing new to import\n")?;
            }
            for (kind, name) in imported {
                emit(out, &format!("imported {} `{name}`\n", kind.label()))?;
            }
        }
        Command::Dash => {
            return Err(BcError::Invalid("`dash` is interactive and cannot run here".into()));
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Kind;
    use std::fs;

    fn setup() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().join("bc"), dir.path().join("home"));
        (dir, paths)
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn text(out: Vec<u8>) -> String {
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn import_then_ls_then_which() {
        let (_d, paths) = setup();
        write(&paths.claude_dir().join("skills/s/SKILL.md"), "---\ndescription: does s\n---\n");
        write(&paths.config_file(), "[profiles.global]\nskills=[\"s\",\"ghost\"]\n");

        let mut out = Vec::new();
        execute(Command::Import, &paths, Path::new("/w"), &mut out).unwrap();
        assert_eq!(text(out), "imported skill `s`\n");

        let mut out = Vec::new();
        execute(Command::Import, &paths, Path::new("/w"), &mut out).unwrap();
        assert_eq!(text(out), "nothing new to import\n");

        let mut out = Vec::new();
        execute(Command::Ls { kind: Some(Kind::Skill) }, &paths, Path::new("/w"), &mut out).unwrap();
        let ls = text(out);
        assert!(ls.contains("does s") && ls.contains("global"));

        let mut out = Vec::new();
        execute(Command::Which, &paths, Path::new("/w"), &mut out).unwrap();
        let which = text(out);
        assert!(which.contains("s (~") && which.contains("ghost (MISSING"));
    }

    #[test]
    fn add_reports_name_and_surfaces_errors() {
        let (d, paths) = setup();
        let f = d.path().join("a.md");
        write(&f, "x");
        let mut out = Vec::new();
        let cmd = Command::Add { kind: Kind::Agent, path: f, name: None };
        execute(cmd, &paths, Path::new("/w"), &mut out).unwrap();
        assert_eq!(text(out), "added agent `a` to the library\n");
        let bad = Command::Add { kind: Kind::Agent, path: d.path().join("nope.md"), name: None };
        assert!(execute(bad, &paths, Path::new("/w"), &mut Vec::new()).is_err());
    }

    #[test]
    fn dash_is_rejected_by_execute() {
        let (_d, paths) = setup();
        assert!(execute(Command::Dash, &paths, Path::new("/w"), &mut Vec::new()).is_err());
    }

    #[test]
    fn dry_run_prints_command_and_warns_about_missing() {
        let (_d, paths) = setup();
        write(&paths.config_file(), "[profiles.global]\nagents=[\"gone\"]\nclaude_md=[\"m\"]\n");
        write(&paths.kind_dir(Kind::ClaudeMd).join("m.md"), "hi");
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let args = vec!["-c".to_string()];
        assert_eq!(dry_run_report(&paths, Path::new("/w"), &args, &mut out, &mut err).unwrap(), 0);
        let line = text(out);
        assert!(line.starts_with("claude --append-system-prompt-file "));
        assert!(line.trim_end().ends_with(" -c"));
        assert!(text(err).contains("agent `gone` not found"));
        assert!(!paths.run_dir().join(std::process::id().to_string()).exists());
    }

    #[test]
    fn emit_surfaces_write_errors() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("closed"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert!(emit(&mut Broken, "x").is_err());
    }
}
