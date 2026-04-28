use crate::app::App;
use crate::ui::{detail_panel, session_table, status_bar};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = if app.show_detail {
        Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area())
    } else {
        Layout::vertical([
            Constraint::Min(5),
            Constraint::Length(0),
            Constraint::Length(1),
        ])
        .split(frame.area())
    };

    session_table::render(frame, chunks[0], &app.sessions, &mut app.table_state);

    if app.show_detail {
        let selected = app.table_state.selected().and_then(|i| app.sessions.get(i));
        detail_panel::render(frame, chunks[1], selected);
    }

    status_bar::render(frame, chunks[2], app.in_tmux);
}
