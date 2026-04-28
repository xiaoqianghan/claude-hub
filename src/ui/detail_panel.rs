use super::truncate_chars;
use crate::model::session::{SessionInfo, SessionState};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect, session: Option<&SessionInfo>) {
    let Some(s) = session else {
        let p = Paragraph::new("No session selected")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(p, area);
        return;
    };

    let home = dirs::home_dir()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();
    let display_cwd = s.cwd.replace(&home, "~");

    let dur = s.duration_since_start();
    let dur_str = if dur.num_hours() > 0 {
        format!("{}h{}m", dur.num_hours(), dur.num_minutes() % 60)
    } else {
        format!("{}m", dur.num_minutes())
    };

    let model_str = s.model.as_deref().unwrap_or("—");
    let tmux_str = s.tmux_session_name.as_deref().unwrap_or("—");
    let branch_str = s.git_branch.as_deref().unwrap_or("—");

    let dim = Style::default().fg(Color::DarkGray);

    let mut lines = vec![
        // Line 1: identity
        Line::from(vec![
            Span::styled("Tmux: ", dim),
            Span::styled(tmux_str, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("Branch: ", dim),
            Span::raw(branch_str),
            Span::raw("  "),
            Span::styled("Model: ", dim),
            Span::raw(model_str.to_string()),
        ]),
        // Line 2: path + timing
        Line::from(vec![Span::styled("CWD: ", dim), Span::raw(display_cwd)]),
        Line::from(vec![
            Span::styled("PID: ", dim),
            Span::raw(format!("{}  ", s.pid)),
            Span::styled("Duration: ", dim),
            Span::raw(format!("{}  ", dur_str)),
            Span::styled("Turns: ", dim),
            Span::raw(format!("{}  ", s.turn_count)),
            Span::styled("Tokens: ", dim),
            Span::raw(s.tokens_display()),
        ]),
        Line::from(""),
    ];

    // Last user prompt
    if let Some(prompt) = &s.last_prompt {
        lines.push(Line::from(vec![Span::styled(
            "You asked: ",
            Style::default().fg(Color::Blue),
        )]));
        let display = truncate_chars(prompt, 200);
        lines.push(Line::from(format!("  {}", display)));
    }

    // Last assistant response
    if let Some(text) = &s.last_assistant_text {
        lines.push(Line::from(""));
        let label = match s.state {
            SessionState::WaitingForInput => "Claude replied: ",
            _ => "Claude (latest): ",
        };
        lines.push(Line::from(vec![Span::styled(
            label,
            Style::default().fg(Color::Green),
        )]));
        let display = truncate_chars(text, 300);
        for line in display.lines().take(6) {
            lines.push(Line::from(format!("  {}", line)));
        }
    }

    let title_style = match s.state {
        SessionState::WaitingForInput => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        SessionState::Working => Style::default().fg(Color::Green),
        SessionState::Stale => Style::default().fg(Color::Red),
    };
    let title = format!(
        " {} {} — {} ",
        s.state.symbol(),
        s.state.label(),
        s.project_name,
    );

    let p = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(title, title_style)),
    );
    frame.render_widget(p, area);
}
