mod state;
mod ui;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::config;
use crate::error::{BcError, Result};
use crate::library::Library;
use crate::paths::Paths;
use crate::sessions;

pub use state::{App, Tab};

/// Result of handling one key press.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
    Reload,
}

/// Pure key → state transition (the event loop only does I/O around it).
pub fn handle_key(app: &mut App, key: KeyCode) -> Action {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => return Action::Quit,
        KeyCode::Right | KeyCode::Tab | KeyCode::Char('l') => app.next_tab(),
        KeyCode::Left | KeyCode::BackTab | KeyCode::Char('h') => app.prev_tab(),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Char(' ') | KeyCode::Enter => app.toggle_membership(),
        KeyCode::Char('p') => app.cycle_profile(),
        KeyCode::Char('r') => return Action::Reload,
        _ => {}
    }
    Action::Continue
}

pub fn run(paths: &Paths) -> Result<()> {
    let cfg = config::load(&paths.config_file())?;
    let library = Library::scan(&paths.library_dir())?;
    let live = sessions::list_alive(&paths.sessions_dir(), &paths.run_dir())?;
    let mut app = App::new(cfg, library, live);

    let mut terminal = ratatui::init();
    let outcome = event_loop(&mut terminal, &mut app, paths);
    ratatui::restore();
    outcome
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App, paths: &Paths) -> Result<()> {
    let tty = |e: std::io::Error| BcError::io("terminal", e);
    loop {
        terminal.draw(|frame| ui::draw(frame, app)).map_err(tty)?;
        if !event::poll(Duration::from_secs(2)).map_err(tty)? {
            app.sessions = sessions::list_alive(&paths.sessions_dir(), &paths.run_dir())?;
            continue;
        }
        let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read().map_err(tty)?
        else {
            continue;
        };
        match handle_key(app, code) {
            Action::Quit => return Ok(()),
            Action::Reload => app.reload(paths)?,
            Action::Continue => {}
        }
        if app.dirty {
            config::save(&paths.config_file(), &app.config)?;
            app.dirty = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::Kind;

    fn app() -> App {
        let library = Library::from_entries(vec![
            crate::library::Entry {
                kind: Kind::Skill,
                name: "a".into(),
                description: None,
                path: "/x".into(),
                size_bytes: 1,
            },
            crate::library::Entry {
                kind: Kind::Skill,
                name: "b".into(),
                description: None,
                path: "/x".into(),
                size_bytes: 1,
            },
        ]);
        App::new(Config::default(), library, vec![])
    }

    #[test]
    fn keys_map_to_actions() {
        let mut a = app();
        assert_eq!(handle_key(&mut a, KeyCode::Char('q')), Action::Quit);
        assert_eq!(handle_key(&mut a, KeyCode::Esc), Action::Quit);
        assert_eq!(handle_key(&mut a, KeyCode::Char('r')), Action::Reload);
        assert_eq!(handle_key(&mut a, KeyCode::Char('z')), Action::Continue);
    }

    #[test]
    fn navigation_and_toggle_keys() {
        let mut a = app();
        handle_key(&mut a, KeyCode::Down);
        assert_eq!(a.selected, 1);
        handle_key(&mut a, KeyCode::Char('k'));
        assert_eq!(a.selected, 0);
        handle_key(&mut a, KeyCode::Char(' '));
        assert!(a.dirty && a.config.is_member("global", Kind::Skill, "a"));
        handle_key(&mut a, KeyCode::Right);
        assert_eq!(a.tab, Tab::Agents);
        handle_key(&mut a, KeyCode::Left);
        assert_eq!(a.tab, Tab::Skills);
        handle_key(&mut a, KeyCode::Char('p'));
        assert!(a.status.contains("active profile"));
    }
}
