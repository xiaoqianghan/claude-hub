use crate::model::feed::{FeedEvent, FeedEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

fn event_style(event: &FeedEvent) -> Style {
    match &event.kind {
        FeedEventKind::StateChanged { to, .. } => to.style(),
        FeedEventKind::SessionStarted => Style::default().fg(Color::Cyan),
        FeedEventKind::SessionEnded => Style::default().fg(Color::Red).add_modifier(Modifier::DIM),
    }
}

pub fn render(frame: &mut Frame, area: Rect, feed: &std::collections::VecDeque<FeedEvent>) {
    let max_visible = area.height.saturating_sub(2) as usize;

    let lines: Vec<Line> = feed
        .iter()
        .rev()
        .take(max_visible)
        .map(|e| {
            let dim = Style::default().fg(Color::DarkGray);
            let style = event_style(e);
            Line::from(vec![
                Span::styled(e.time_display(), dim),
                Span::raw("  "),
                Span::styled(e.state_symbol(), style),
                Span::raw(" "),
                Span::styled(
                    super::truncate_chars(&e.project_name, 20),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(e.description(), style),
            ])
        })
        .collect();

    let title = if feed.is_empty() {
        " Events ".to_string()
    } else {
        format!(" Events ({}) ", feed.len())
    };

    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(p, area);
}
