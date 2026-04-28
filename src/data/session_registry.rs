use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryEntry {
    pub pid: u32,
    pub session_id: String,
    pub cwd: String,
    pub started_at: u64,
    pub version: String,
    #[allow(dead_code)]
    pub kind: Option<String>,
    #[allow(dead_code)]
    pub entrypoint: Option<String>,
}

pub async fn scan_sessions(dir: &Path) -> Result<Vec<RegistryEntry>> {
    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let Ok(content) = tokio::fs::read_to_string(&path).await else {
                continue;
            };
            let Ok(reg) = serde_json::from_str::<RegistryEntry>(&content) else {
                continue;
            };
            entries.push(reg);
        }
    }
    Ok(entries)
}
