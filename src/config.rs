use std::path::PathBuf;

pub fn claude_dir() -> PathBuf {
    dirs::home_dir().expect("no home directory").join(".claude")
}

pub fn sessions_dir() -> PathBuf {
    claude_dir().join("sessions")
}

pub fn projects_dir() -> PathBuf {
    claude_dir().join("projects")
}

pub fn encode_cwd(cwd: &str) -> String {
    cwd.chars()
        .map(|c| if c == '/' || c == '.' { '-' } else { c })
        .collect()
}

#[allow(dead_code)]
pub fn project_dir(cwd: &str) -> PathBuf {
    projects_dir().join(encode_cwd(cwd))
}
