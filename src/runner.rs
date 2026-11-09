use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::compose::{Composed, compose};
use crate::config;
use crate::error::{BcError, Result};
use crate::launch;
use crate::library::Library;
use crate::paths::Paths;
use crate::resolve::{ProfileRequest, Resolved, resolve};
use crate::sessions::{self, SessionRecord};

/// Everything needed to start (or just print) a composed session.
#[derive(Debug)]
pub struct Prepared {
    pub resolved: Resolved,
    pub composed: Composed,
    pub args: Vec<OsString>,
    pub session_dir: PathBuf,
}

/// Resolves the profile set for `cwd` plus the explicitly requested `profiles`
/// and materialises it under `<run_dir>/<pid>`.
pub fn prepare(
    paths: &Paths,
    cwd: &Path,
    pid: u32,
    request: &ProfileRequest,
    user_args: &[String],
) -> Result<Prepared> {
    let cfg = config::load(&paths.config_file())?;
    if request.exclusive && request.names.is_empty() {
        return Err(BcError::Invalid("--exclusive requires --profile".into()));
    }
    if let Some(unknown) = request
        .names
        .iter()
        .find(|p| !cfg.profiles.contains_key(*p))
    {
        return Err(BcError::Invalid(format!(
            "unknown profile `{unknown}` (see [profiles.*] in {})",
            paths.config_file().display()
        )));
    }
    let library = Library::load(paths)?;
    let resolved = resolve(&cfg, cwd, &paths.home, request);
    let session_dir = paths.run_dir().join(pid.to_string());
    if session_dir.exists() {
        fs::remove_dir_all(&session_dir).map_err(|e| BcError::io(&session_dir, e))?;
    }
    let composed = compose(&resolved, &library, &session_dir)?;
    let args = launch::build_args(&composed, user_args);
    Ok(Prepared {
        resolved,
        composed,
        args,
        session_dir,
    })
}

pub fn record_for(prepared: &Prepared, cwd: &Path, pid: u32) -> SessionRecord {
    SessionRecord {
        pid,
        cwd: cwd.to_string_lossy().into_owned(),
        profiles: prepared.resolved.profile_names(),
        skills: prepared.resolved.skills.clone(),
        agents: prepared.resolved.agents.clone(),
        claude_md: prepared.resolved.claude_md.clone(),
        plugins: prepared.resolved.plugins.clone(),
        mcp: prepared.resolved.mcp.clone(),
        started_unix: sessions::now_unix(),
    }
}

/// Full lifecycle: compose, register session, run claude, always clean up.
/// Returns claude's exit code.
pub fn run_session(
    paths: &Paths,
    cwd: &Path,
    request: &ProfileRequest,
    user_args: &[String],
    bin: &str,
) -> Result<i32> {
    let pid = std::process::id();
    let prepared = prepare(paths, cwd, pid, request, user_args)?;
    let record_file = sessions::write(&paths.sessions_dir(), &record_for(&prepared, cwd, pid))?;
    let outcome = launch::run(bin, &prepared.args);
    let cleanup = cleanup(&record_file, &prepared.session_dir);
    let code = outcome?;
    cleanup?;
    Ok(code)
}

fn cleanup(record_file: &Path, session_dir: &Path) -> Result<()> {
    sessions::remove(record_file)?;
    match fs::remove_dir_all(session_dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(BcError::io(session_dir, e)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn setup() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().join("bc"), dir.path().join("home"));
        write(
            &paths.config_file(),
            "[profiles.global]\nskills=[\"s\"]\nclaude_md=[\"m\"]\n",
        );
        write(
            &paths.kind_dir(crate::types::Kind::Skill).join("s/SKILL.md"),
            "s",
        );
        write(
            &paths.kind_dir(crate::types::Kind::ClaudeMd).join("m.md"),
            "m",
        );
        (dir, paths)
    }

    #[test]
    fn exclusive_needs_a_profile() {
        let (_d, paths) = setup();
        let request = ProfileRequest {
            names: vec![],
            exclusive: true,
        };
        let err = prepare(&paths, Path::new("/any"), 5, &request, &[]).unwrap_err();
        assert!(err.to_string().contains("--exclusive requires --profile"));
    }

    #[test]
    fn prepare_composes_and_builds_args() {
        let (_d, paths) = setup();
        let p = prepare(
            &paths,
            Path::new("/any"),
            7,
            &ProfileRequest::default(),
            &["-c".into()],
        )
        .unwrap();
        assert_eq!(p.session_dir, paths.run_dir().join("7"));
        assert_eq!(p.args.len(), 5);
        assert_eq!(p.args.last().unwrap(), "-c");
        assert!(p.composed.plugin_dir.unwrap().join("skills/s").exists());
    }

    #[test]
    fn prepare_replaces_leftover_session_dir() {
        let (_d, paths) = setup();
        prepare(
            &paths,
            Path::new("/any"),
            7,
            &ProfileRequest::default(),
            &[],
        )
        .unwrap();
        assert!(
            prepare(
                &paths,
                Path::new("/any"),
                7,
                &ProfileRequest::default(),
                &[]
            )
            .is_ok()
        );
    }

    #[test]
    fn record_reflects_resolution() {
        let (_d, paths) = setup();
        let p = prepare(
            &paths,
            Path::new("/any"),
            9,
            &ProfileRequest::default(),
            &[],
        )
        .unwrap();
        let r = record_for(&p, Path::new("/any"), 9);
        assert_eq!((r.pid, r.cwd.as_str()), (9, "/any"));
        assert_eq!(r.profiles, ["global"]);
        assert_eq!(r.skills, ["s"]);
    }

    #[test]
    fn run_session_returns_exit_code_and_cleans_up() {
        let (_d, paths) = setup();
        let code = run_session(
            &paths,
            Path::new("/any"),
            &ProfileRequest::default(),
            &[],
            "true",
        )
        .unwrap();
        assert_eq!(code, 0);
        assert!(sessions::list(&paths.sessions_dir()).unwrap().is_empty());
        assert!(
            !paths
                .run_dir()
                .join(std::process::id().to_string())
                .exists()
        );
    }

    #[test]
    fn run_session_cleans_up_even_when_launch_fails() {
        let (_d, paths) = setup();
        let err = run_session(
            &paths,
            Path::new("/any"),
            &ProfileRequest::default(),
            &[],
            "definitely-not-a-binary-bc",
        );
        assert!(matches!(err, Err(BcError::Launch { .. })));
        assert!(sessions::list(&paths.sessions_dir()).unwrap().is_empty());
    }

    #[test]
    fn cleanup_is_idempotent() {
        let (_d, paths) = setup();
        let pid = std::process::id();
        let p = prepare(
            &paths,
            Path::new("/any"),
            pid,
            &ProfileRequest::default(),
            &[],
        )
        .unwrap();
        let rec = sessions::write(
            &paths.sessions_dir(),
            &record_for(&p, Path::new("/any"), pid),
        )
        .unwrap();
        cleanup(&rec, &p.session_dir).unwrap();
        assert!(!rec.exists() && !p.session_dir.exists());
        cleanup(&rec, &p.session_dir).unwrap();
    }
}
