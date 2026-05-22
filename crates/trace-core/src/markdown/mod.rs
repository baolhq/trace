pub mod doc;
pub mod links;
pub mod parse;
pub mod serialize;

/// Extracts the document title from Markdown content.
/// Uses the first `# H1` line, falling back to the filename stem.
pub fn extract_title(content: &str, rel_path: &str) -> String {
    content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l[2..].trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::path::Path::new(rel_path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel_path.to_string())
        })
}
