use std::collections::HashSet;
use std::fs;
use std::path::Path;

static BUILTIN_IGNORES: &[&str] = &[".git", "target", "node_modules"];

#[derive(Debug, Clone)]
pub struct IgnoreMatcher {
    names: HashSet<String>,
}

impl IgnoreMatcher {
    pub fn new(src_root: &Path) -> Self {
        let mut names: HashSet<String> = BUILTIN_IGNORES.iter().map(|s| s.to_string()).collect();

        let ignore_file = src_root.join(".syncdignore");
        if let Ok(contents) = fs::read_to_string(&ignore_file) {
            for line in contents.lines() {
                let line = line.trim();

                if line.is_empty() || line.starts_with("#") {
                    continue;
                }

                names.insert(line.to_string());
            }
        }

        IgnoreMatcher { names }
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .map(|s| self.names.contains(s))
                .unwrap_or(false)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn ignore_git_directory() {
        let p = PathBuf::from("/repo/.git/config");
        let ignore = IgnoreMatcher::new(&p);

        assert!(ignore.is_ignored(&p));
    }

    #[test]
    fn ignores_nested_target_directory() {
        let p = PathBuf::from("/repo/crate/target/debug/app");
        let ignore = IgnoreMatcher::new(&p);

        assert!(ignore.is_ignored(&p));
    }

    #[test]
    fn does_not_ignore_normal_paths() {
        let p = PathBuf::from("/repo/src/main.rs");
        let ignore = IgnoreMatcher::new(&p);

        assert!(!ignore.is_ignored(&p));
    }

    #[test]
    fn loads_ignore_file_and_builtin_rules() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::write(root.join(".syncdignore"), "dist\n# comment\nbuild\n").unwrap();

        let ignore = IgnoreMatcher::new(root);

        assert!(ignore.is_ignored(&root.join("dist/file.txt")));
        assert!(ignore.is_ignored(&root.join("build/out")));
        assert!(ignore.is_ignored(&root.join(".git/config")));
        assert!(!ignore.is_ignored(&root.join("src/main.rs")));
    }
}
