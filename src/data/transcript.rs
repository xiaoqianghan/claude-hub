use crate::config;
use crate::model::event::TranscriptLine;
use crate::model::session::SessionState;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct TranscriptSummary {
    pub state: SessionState,
    pub last_timestamp: Option<String>,
    pub last_prompt: Option<String>,
    pub last_assistant_text: Option<String>,
    pub model: Option<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub turn_count: u32,
    pub git_branch: Option<String>,
}

fn find_transcript_file(cwd: &str, session_id: &str) -> Option<PathBuf> {
    let project_dir = config::projects_dir().join(config::encode_cwd(cwd));
    let direct = project_dir.join(format!("{}.jsonl", session_id));
    if direct.is_file() {
        return Some(direct);
    }

    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in std::fs::read_dir(&project_dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonl")
            && let Some(mt) = entry.metadata().ok().and_then(|m| m.modified().ok())
            && best.as_ref().is_none_or(|(_, t)| mt > *t)
        {
            best = Some((path, mt));
        }
    }
    best.map(|(p, _)| p)
}

fn read_tail(path: &Path, max_bytes: u64) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    let seeked = len > max_bytes;
    if seeked {
        file.seek(SeekFrom::End(-(max_bytes as i64)))?;
    }
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let start = if seeked {
        buf.iter()
            .position(|&b| b == b'\n')
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };
    Ok(String::from_utf8_lossy(&buf[start..]).into_owned())
}

fn extract_text(content: &serde_json::Value) -> Option<String> {
    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for item in arr {
            if item.get("type").and_then(|t| t.as_str()) == Some("text")
                && let Some(t) = item.get("text").and_then(|t| t.as_str())
            {
                return Some(t.to_string());
            }
        }
    }
    None
}

pub fn analyze(cwd: &str, session_id: &str) -> TranscriptSummary {
    let default = TranscriptSummary {
        state: SessionState::Working,
        last_timestamp: None,
        last_prompt: None,
        last_assistant_text: None,
        model: None,
        total_input_tokens: 0,
        total_output_tokens: 0,
        turn_count: 0,
        git_branch: None,
    };

    let Some(path) = find_transcript_file(cwd, session_id) else {
        return default;
    };

    let tail = match read_tail(&path, 32768) {
        Ok(t) => t,
        Err(_) => return default,
    };

    let mut last_meaningful_event: Option<&str> = None;
    let mut last_stop_reason: Option<String> = None;
    let mut last_prompt: Option<String> = None;
    let mut last_assistant_text: Option<String> = None;
    let mut model: Option<String> = None;
    let mut total_in: u64 = 0;
    let mut total_out: u64 = 0;
    let mut turns: u32 = 0;
    let mut last_ts: Option<String> = None;
    let mut git_branch: Option<String> = None;

    for evt in tail
        .lines()
        .filter_map(|line| serde_json::from_str::<TranscriptLine>(line).ok())
    {
        if let Some(ts) = &evt.timestamp {
            last_ts = Some(ts.clone());
        }
        if let Some(br) = &evt.git_branch {
            git_branch = Some(br.clone());
        }

        match evt.event_type.as_str() {
            "last-prompt" => {
                if let Some(p) = &evt.last_prompt {
                    last_prompt = Some(p.clone());
                }
            }
            "assistant" => {
                last_meaningful_event = Some("assistant");
                if let Some(msg) = &evt.message {
                    if let Some(m) = &msg.model {
                        model = Some(m.clone());
                    }
                    if let Some(usage) = &msg.usage {
                        total_in += usage.input_tokens.unwrap_or(0);
                        total_out += usage.output_tokens.unwrap_or(0);
                    }
                    last_stop_reason = msg.stop_reason.clone();
                    if let Some(content) = &msg.content
                        && let Some(text) = extract_text(content)
                        && !text.is_empty()
                    {
                        last_assistant_text = Some(text);
                    }
                }
            }
            "user" => {
                if let Some(msg) = &evt.message
                    && msg.role.as_deref() == Some("user")
                {
                    if msg.content.as_ref().is_some_and(|c| c.is_string()) {
                        turns += 1;
                    }
                    // tool_result events have role=user but array content
                    let is_tool_result = msg
                        .content
                        .as_ref()
                        .and_then(|c| c.as_array())
                        .is_some_and(|arr| {
                            arr.iter().any(|v| {
                                v.get("type").and_then(|t| t.as_str()) == Some("tool_result")
                            })
                        });
                    if is_tool_result {
                        last_meaningful_event = Some("tool_result");
                    } else {
                        last_meaningful_event = Some("user");
                    }
                }
            }
            "system" if evt.subtype.as_deref() == Some("stop_hook_summary") => {
                last_meaningful_event = Some("stop");
            }
            _ => {}
        }
    }

    let state = match last_meaningful_event {
        Some("stop") => SessionState::WaitingForInput,
        Some("assistant") => match last_stop_reason.as_deref() {
            Some("end_turn") => SessionState::WaitingForInput,
            _ => SessionState::Working,
        },
        Some("user") => SessionState::Working,
        Some("tool_result") => SessionState::Working,
        _ => SessionState::Working,
    };

    TranscriptSummary {
        state,
        last_timestamp: last_ts,
        last_prompt,
        last_assistant_text,
        model,
        total_input_tokens: total_in,
        total_output_tokens: total_out,
        turn_count: turns,
        git_branch,
    }
}
