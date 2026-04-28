use crate::data::{session_registry, tmux, transcript};
use crate::model::session::{SessionInfo, SessionState};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub async fn build_session_list() -> Result<Vec<SessionInfo>> {
    let sessions_dir = crate::config::sessions_dir();
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = session_registry::scan_sessions(&sessions_dir).await?;
    let panes = tmux::list_panes().await?;

    let pids: Vec<u32> = entries.iter().map(|e| e.pid).collect();
    let ppid_map = tmux::get_ppid_map(&pids).await?;

    let pane_by_pid: HashMap<u32, &tmux::TmuxPane> =
        panes.iter().map(|p| (p.pane_pid, p)).collect();

    let mut sessions = Vec::new();

    for entry in &entries {
        let alive = ppid_map.contains_key(&entry.pid);

        let tmux_pane = ppid_map
            .get(&entry.pid)
            .and_then(|ppid| pane_by_pid.get(ppid))
            .copied();

        let summary = transcript::analyze(&entry.cwd, &entry.session_id);

        let state = if !alive {
            SessionState::Stale
        } else {
            summary.state
        };

        let last_activity = summary
            .last_timestamp
            .as_deref()
            .and_then(|ts| ts.parse::<DateTime<Utc>>().ok());

        let project_name = entry
            .cwd
            .rsplit('/')
            .next()
            .unwrap_or(&entry.cwd)
            .to_string();

        sessions.push(SessionInfo {
            pid: entry.pid,
            session_id: entry.session_id.clone(),
            cwd: entry.cwd.clone(),
            started_at: entry.started_at,
            version: entry.version.clone(),
            tmux_target: tmux_pane.map(|p| p.target.clone()),
            tmux_session_name: tmux_pane.map(|p| p.session_name.clone()),
            state,
            last_activity,
            last_prompt: summary.last_prompt,
            last_assistant_text: summary.last_assistant_text,
            model: summary.model,
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            turn_count: summary.turn_count,
            git_branch: summary.git_branch,
            project_name,
        });
    }

    Ok(sessions)
}
