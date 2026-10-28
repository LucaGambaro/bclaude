use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs, Wrap};

use super::state::{App, Tab};

fn headers(tab: Tab) -> [&'static str; 4] {
    match tab {
        Tab::Skills | Tab::Agents | Tab::ClaudeMd => ["", "Name", "Profiles", "Description"],
        Tab::Profiles => ["", "Profile", "Counts", "Match"],
        Tab::Sessions => ["", "Directory", "Process", "Profiles"],
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let [tabs_area, body, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(2)])
        .areas(frame.area());
    let [list_area, preview_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .areas(body);

    let titles: Vec<&str> = Tab::ALL.iter().map(|t| t.title()).collect();
    let selected_tab = Tab::ALL.iter().position(|t| *t == app.tab).unwrap_or(0);
    frame.render_widget(
        Tabs::new(titles)
            .select(selected_tab)
            .block(Block::default().borders(Borders::ALL).title(" betterClaude "))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        tabs_area,
    );

    let rows: Vec<Row> = app
        .rows()
        .into_iter()
        .map(|r| {
            let mark = if r.member { "●" } else { " " };
            let style = if r.member { Style::default().fg(Color::Green) } else { Style::default() };
            Row::new([Cell::from(mark), Cell::from(r.name), Cell::from(r.extra), Cell::from(r.detail)]).style(style)
        })
        .collect();
    let table = Table::new(
        rows,
        [Constraint::Length(2), Constraint::Percentage(30), Constraint::Percentage(22), Constraint::Min(10)],
    )
    .header(Row::new(headers(app.tab)).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL))
    .row_highlight_style(Style::default().bg(Color::DarkGray));
    let mut state = TableState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(table, list_area, &mut state);

    frame.render_widget(
        Paragraph::new(app.preview())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title(" Preview ")),
        preview_area,
    );

    frame.render_widget(
        Paragraph::new(format!(
            "profile: {}  |  ←→ tab  ↑↓ move  space toggle  p profile  r reload  q quit\n{}",
            app.active_profile, app.status
        )),
        footer,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::library::Library;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(app: &App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        buffer.content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn renders_tabs_footer_and_rows() {
        let mut config = Config::default();
        config.set_membership("global", crate::types::Kind::Skill, "x", true);
        let library = Library::from_entries(vec![crate::library::Entry {
            kind: crate::types::Kind::Skill,
            name: "x".into(),
            description: Some("desc here".into()),
            path: "/nonexistent".into(),
            size_bytes: 1,
        }]);
        let mut app = App::new(config, library, vec![]);
        let out = render(&app);
        assert!(out.contains("betterClaude"));
        assert!(out.contains("Skills") && out.contains("Sessions"));
        assert!(out.contains("desc here"));
        assert!(out.contains("profile: global"));
        assert!(out.contains("●"));
        for tab in Tab::ALL {
            app.tab = tab;
            assert!(render(&app).contains(headers(tab)[1]));
        }
    }
}
