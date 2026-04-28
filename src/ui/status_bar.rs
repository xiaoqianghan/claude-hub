use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, area: Rect, in_tmux: bool) {
    let key = Style::default().fg(Color::Cyan);
    let mut spans = vec![Span::styled(" ↑↓/jk ", key), Span::raw("navigate  ")];

    if in_tmux {
        spans.push(Span::styled("Enter ", key));
        spans.push(Span::raw("switch  "));
    }

    spans.extend([
        Span::styled("r ", key),
        Span::raw("refresh  "),
        Span::styled("s ", key),
        Span::raw("sort  "),
        Span::styled("Tab ", key),
        Span::raw("detail  "),
        Span::styled("q ", key),
        Span::raw("quit"),
    ]);

    let p = Paragraph::new(Line::from(spans));
    frame.render_widget(p, area);
}
