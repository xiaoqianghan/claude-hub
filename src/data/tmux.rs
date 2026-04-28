use anyhow::Result;
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TmuxPane {
    pub session_name: String,
    pub pane_pid: u32,
    pub current_command: String,
    pub current_path: String,
    pub target: String,
}

pub async fn list_panes() -> Result<Vec<TmuxPane>> {
    let output = Command::new("tmux")
        .args([
            "list-panes",
            "-a",
            "-F",
            "#{session_name}\t#{window_index}\t#{pane_index}\t#{pane_pid}\t#{pane_current_command}\t#{pane_current_path}",
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut panes = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 6 {
            continue;
        }
        let Some(pane_pid) = parts[3].parse::<u32>().ok() else {
            continue;
        };
        panes.push(TmuxPane {
            session_name: parts[0].to_string(),
            pane_pid,
            current_command: parts[4].to_string(),
            current_path: parts[5].to_string(),
            target: format!("{}:{}.{}", parts[0], parts[1], parts[2]),
        });
    }

    Ok(panes)
}

pub async fn get_ppid_map(pids: &[u32]) -> Result<HashMap<u32, u32>> {
    if pids.is_empty() {
        return Ok(HashMap::new());
    }
    let pid_args: String = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let output = Command::new("ps")
        .args(["-o", "pid=,ppid=", "-p", &pid_args])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map = HashMap::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2
            && let (Ok(pid), Ok(ppid)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
        {
            map.insert(pid, ppid);
        }
    }
    Ok(map)
}

pub async fn switch_client(target: &str) -> Result<()> {
    // target is "session:window.pane", switch to session then select the exact pane
    Command::new("tmux")
        .args(["switch-client", "-t", target])
        .status()
        .await?;
    Command::new("tmux")
        .args(["select-pane", "-t", target])
        .status()
        .await?;
    Ok(())
}

pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}
