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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn ignore_git_directory() {
        let p = PathBuf::from("/repo/.git/config");
        assert!(is_ignored(&p));
    }

    #[test]
    fn ignores_nested_target_directory() {
        let p = PathBuf::from("/repo/crate/target/debug/app");
        assert!(is_ignored(&p));
    }

    #[test]
    fn does_not_ignore_normal_paths() {
        let p = PathBuf::from("/repo/src/main.rs");
        assert!(!is_ignored(&p));
    }
}
