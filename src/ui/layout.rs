use crate::app::App;
use crate::ui::{detail_panel, event_feed, session_table, status_bar};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::Widget;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(7),
        Constraint::Length(1),
    ])
    .split(frame.area());

    session_table::render(frame, chunks[0], &app.sessions, &mut app.table_state);
    event_feed::render(frame, chunks[1], &app.feed);
    status_bar::render(frame, chunks[2], app.in_tmux);

    if app.show_detail {
        let selected = app.table_state.selected().and_then(|i| app.sessions.get(i));
        let overlay = center_rect(frame.area(), 70, 60);
        ratatui::widgets::Clear.render(overlay, frame.buffer_mut());
        detail_panel::render(frame, overlay, selected);
    }
}

fn center_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
