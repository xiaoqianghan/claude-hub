use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Working,
    WaitingForInput,
    Stale,
}

impl SessionState {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Working => "●",
            Self::WaitingForInput => "◆",
            Self::Stale => "✕",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Working => "Running",
            Self::WaitingForInput => "Needs you",
            Self::Stale => "Stale",
        }
    }

    pub fn needs_action(&self) -> bool {
        matches!(self, Self::WaitingForInput)
    }

    pub fn sort_priority(&self) -> u8 {
        match self {
            Self::WaitingForInput => 0,
            Self::Working => 1,
            Self::Stale => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub pid: u32,
    pub session_id: String,
    pub cwd: String,
    pub started_at: u64,
    #[allow(dead_code)]
    pub version: String,

    pub tmux_target: Option<String>,
    pub tmux_session_name: Option<String>,

    pub state: SessionState,
    pub last_activity: Option<DateTime<Utc>>,
    pub last_prompt: Option<String>,
    pub last_assistant_text: Option<String>,
    pub model: Option<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub turn_count: u32,
    pub git_branch: Option<String>,

    pub project_name: String,
}

impl SessionInfo {
    pub fn duration_since_start(&self) -> chrono::Duration {
        let started = DateTime::from_timestamp_millis(self.started_at as i64).unwrap_or(Utc::now());
        Utc::now() - started
    }

    pub fn idle_duration(&self) -> String {
        let Some(last) = self.last_activity else {
            return "—".to_string();
        };
        let delta = Utc::now() - last;
        if delta.num_seconds() < 10 {
            "just now".to_string()
        } else if delta.num_seconds() < 60 {
            format!("{}s", delta.num_seconds())
        } else if delta.num_minutes() < 60 {
            format!("{}m", delta.num_minutes())
        } else {
            format!("{}h{}m", delta.num_hours(), delta.num_minutes() % 60)
        }
    }

    pub fn tokens_display(&self) -> String {
        fn fmt(n: u64) -> String {
            if n >= 1_000_000 {
                format!("{:.1}M", n as f64 / 1_000_000.0)
            } else if n >= 1000 {
                format!("{}k", n / 1000)
            } else {
                n.to_string()
            }
        }
        format!(
            "{}/{}",
            fmt(self.total_input_tokens),
            fmt(self.total_output_tokens)
        )
    }
}
