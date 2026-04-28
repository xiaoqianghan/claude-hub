use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TranscriptLine {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: Option<String>,
    pub message: Option<MessagePayload>,
    pub subtype: Option<String>,
    #[serde(rename = "lastPrompt")]
    pub last_prompt: Option<String>,
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePayload {
    pub role: Option<String>,
    pub model: Option<String>,
    pub stop_reason: Option<String>,
    pub content: Option<serde_json::Value>,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}
