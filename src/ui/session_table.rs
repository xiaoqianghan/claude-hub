use crate::model::session::SessionInfo;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    sessions: &[SessionInfo],
    state: &mut TableState,
) {
    let header = Row::new(vec![
        Cell::from(" #"),
        Cell::from("Status"),
        Cell::from("Tmux"),
        Cell::from("Idle"),
        Cell::from("Context"),
    ])
    .style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan),
    );

    let rows: Vec<Row> = sessions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let status_text = format!("{} {}", s.state.symbol(), s.state.label());
            let style = s.state.style();

            let tmux_label = s.tmux_target.as_deref().unwrap_or("—").to_string();

            Row::new(vec![
                Cell::from(format!("{:>2}", i + 1)),
                Cell::from(Span::styled(status_text, style)),
                Cell::from(tmux_label),
                Cell::from(s.idle_duration()),
                Cell::from(s.tokens_display()),
            ])
        })
        .collect();

    let needs_action = sessions.iter().filter(|s| s.state.needs_action()).count();
    let total = sessions.len();
    let title = if needs_action > 0 {
        format!(
            " Claude Hub — {} need{} you / {} total ",
            needs_action,
            if needs_action == 1 { "s" } else { "" },
            total
        )
    } else {
        format!(
            " Claude Hub — {} session{}, all running ",
            total,
            if total == 1 { "" } else { "s" }
        )
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_stateful_widget(table, area, state);
}
