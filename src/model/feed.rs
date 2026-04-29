use chrono::{DateTime, Utc};

use crate::model::session::SessionState;

#[derive(Debug, Clone)]
pub struct FeedEvent {
    pub timestamp: DateTime<Utc>,
    pub project_name: String,
    pub kind: FeedEventKind,
}

#[derive(Debug, Clone)]
pub enum FeedEventKind {
    StateChanged {
        from: SessionState,
        to: SessionState,
    },
    SessionStarted,
    SessionEnded,
}

impl FeedEvent {
    pub fn description(&self) -> String {
        match &self.kind {
            FeedEventKind::StateChanged { from, to } => {
                format!("{} → {}", from.label(), to.label())
            }
            FeedEventKind::SessionStarted => "Session started".to_string(),
            FeedEventKind::SessionEnded => "Session ended".to_string(),
        }
    }

    pub fn state_symbol(&self) -> &'static str {
        match &self.kind {
            FeedEventKind::StateChanged { to, .. } => to.symbol(),
            FeedEventKind::SessionStarted => "●",
            FeedEventKind::SessionEnded => "✕",
        }
    }

    pub fn time_display(&self) -> String {
        self.timestamp.format("%H:%M").to_string()
    }
}
