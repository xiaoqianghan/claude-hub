use crate::config;
use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

pub enum WatchEvent {
    SessionsChanged,
}

pub async fn watch_sessions(tx: mpsc::UnboundedSender<WatchEvent>) -> Result<()> {
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                    let _ = notify_tx.send(());
                }
                _ => {}
            }
        }
    })?;

    let sessions_dir = config::sessions_dir();
    if sessions_dir.exists() {
        watcher.watch(&sessions_dir, RecursiveMode::NonRecursive)?;
    }

    while let Some(()) = notify_rx.recv().await {
        let _ = tx.send(WatchEvent::SessionsChanged);
    }

    drop(watcher);
    Ok(())
}
