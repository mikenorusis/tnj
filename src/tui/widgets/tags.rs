/// Parse tags from a comma-separated string
/// Returns a vector of trimmed, non-empty tag strings
pub fn parse_tags(tags: Option<&String>) -> Vec<String> {
    match tags {
        Some(tags_str) if !tags_str.trim().is_empty() => {
            tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Format tags as a string with brackets: [tag1] [tag2] [tag3]
pub fn format_tags_brackets(tags: &[String]) -> String {
    tags.iter()
        .map(|tag| format!("[{}]", tag))
        .collect::<Vec<_>>()
        .join(" ")
}

