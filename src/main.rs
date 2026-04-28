mod app;
mod config;
mod data;
mod model;
mod ui;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use data::watcher;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use tokio::sync::mpsc;

enum AppEvent {
    Key(event::KeyEvent),
    SessionsChanged,
    Tick,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal).await;

    terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let mut app = App::new().await?;
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

    // Spawn file watcher with debouncing
    let watch_tx = tx.clone();
    tokio::spawn(async move {
        let (wtx, mut wrx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let _ = watcher::watch_sessions(wtx).await;
        });
        while wrx.recv().await.is_some() {
            let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(500);
            while let Ok(Some(_)) = tokio::time::timeout_at(deadline, wrx.recv()).await {}
            let _ = watch_tx.send(AppEvent::SessionsChanged);
        }
    });

    // Spawn tick timer (refresh every 5s for elapsed time display)
    let tick_tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            if tick_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    // Spawn crossterm input reader
    let input_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            match tokio::task::spawn_blocking(event::read).await {
                Ok(Ok(Event::Key(key))) => {
                    if input_tx.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
                Ok(Ok(_)) => {}
                _ => break,
            }
        }
    });

    loop {
        terminal.draw(|frame| ui::layout::render(frame, &mut app))?;

        match rx.recv().await {
            Some(AppEvent::Key(key)) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                        break;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char('r') => {
                        app.refresh().await?;
                    }
                    KeyCode::Char('s') => app.cycle_sort(),
                    KeyCode::Tab => {
                        app.show_detail = !app.show_detail;
                    }
                    KeyCode::Enter => {
                        if app.in_tmux
                            && let Some(target) =
                                app.selected_session().and_then(|s| s.tmux_target.clone())
                        {
                            data::tmux::switch_client(&target).await?;
                        }
                    }
                    _ => {}
                }
            }
            Some(AppEvent::SessionsChanged) => {
                app.refresh().await?;
            }
            Some(AppEvent::Tick) => {
                // Just re-render to update elapsed/idle times — no data refresh needed
            }
            None => break,
        }
    }

    Ok(())
}
