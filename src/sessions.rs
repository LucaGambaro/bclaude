use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{BcError, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub pid: u32,
    pub cwd: String,
    pub profiles: Vec<String>,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub claude_md: Vec<String>,
    pub started_unix: u64,
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn record_path(dir: &Path, pid: u32) -> PathBuf {
    dir.join(format!("{pid}.json"))
}

pub fn write(dir: &Path, record: &SessionRecord) -> Result<PathBuf> {
    fs::create_dir_all(dir).map_err(|e| BcError::io(dir, e))?;
    let path = record_path(dir, record.pid);
    let json = serde_json::to_string_pretty(record)?;
    fs::write(&path, json).map_err(|e| BcError::io(&path, e))?;
    Ok(path)
}

/// Removing an already-missing record is fine.
pub fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(BcError::io(path, e)),
        _ => Ok(()),
    }
}

/// Unparsable records are skipped; result is sorted by start time.
pub fn list(dir: &Path) -> Result<Vec<SessionRecord>> {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(BcError::io(dir, e)),
    };
    let mut records: Vec<SessionRecord> = read
        .filter_map(|i| i.ok())
        .filter_map(|i| fs::read_to_string(i.path()).ok())
        .filter_map(|text| serde_json::from_str(&text).ok())
        .collect();
    records.sort_by_key(|r| (r.started_unix, r.pid));
    Ok(records)
}

pub fn is_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Lists records whose process is still running, deleting stale record files and run dirs.
pub fn list_alive(sessions_dir: &Path, run_dir: &Path) -> Result<Vec<SessionRecord>> {
    let mut alive = Vec::new();
    for record in list(sessions_dir)? {
        if is_alive(record.pid) {
            alive.push(record);
        } else {
            remove(&record_path(sessions_dir, record.pid))?;
            let stale = run_dir.join(record.pid.to_string());
            if stale.exists() {
                fs::remove_dir_all(&stale).map_err(|e| BcError::io(&stale, e))?;
            }
        }
    }
    Ok(alive)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(pid: u32, started: u64) -> SessionRecord {
        SessionRecord {
            pid,
            cwd: "/x".into(),
            profiles: vec!["global".into()],
            skills: vec!["s".into()],
            agents: vec![],
            claude_md: vec![],
            started_unix: started,
        }
    }

    #[test]
    fn write_list_remove_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        let p2 = write(&sessions, &record(2, 20)).unwrap();
        write(&sessions, &record(1, 10)).unwrap();
        fs::write(sessions.join("junk.json"), "not json").unwrap();
        let listed = list(&sessions).unwrap();
        assert_eq!(listed.iter().map(|r| r.pid).collect::<Vec<_>>(), [1, 2]);
        remove(&p2).unwrap();
        remove(&p2).unwrap();
        assert_eq!(list(&sessions).unwrap().len(), 1);
    }

    #[test]
    fn list_missing_dir_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(list(&dir.path().join("none")).unwrap().is_empty());
    }

    #[test]
    fn current_process_is_alive_and_bogus_pid_is_not() {
        assert!(is_alive(std::process::id()));
        assert!(!is_alive(4_000_000_000));
    }

    #[test]
    fn list_alive_prunes_stale_records_and_run_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        let run = dir.path().join("run");
        let me = std::process::id();
        write(&sessions, &record(me, 1)).unwrap();
        write(&sessions, &record(4_000_000_000, 2)).unwrap();
        fs::create_dir_all(run.join("4000000000")).unwrap();
        let alive = list_alive(&sessions, &run).unwrap();
        assert_eq!(alive.iter().map(|r| r.pid).collect::<Vec<_>>(), [me]);
        assert!(!record_path(&sessions, 4_000_000_000).exists());
        assert!(!run.join("4000000000").exists());
    }

    #[test]
    fn now_is_after_2020() {
        assert!(now_unix() > 1_577_836_800);
    }
}
