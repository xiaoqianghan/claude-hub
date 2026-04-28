pub mod detail_panel;
pub mod layout;
pub mod session_table;
pub mod status_bar;

pub fn truncate_chars(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max {
        let mut result: String = s.chars().take(max - 1).collect();
        result.push('\u{2026}');
        result
    } else {
        s.to_string()
    }
}
