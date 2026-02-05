use std::path::Path;

static IGNORED_NAMES: &[&str] = &[".git", "target", "node_modules"];

pub fn is_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| IGNORED_NAMES.contains(&s))
            .unwrap_or(false)
    })
}
