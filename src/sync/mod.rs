mod engine;
pub use engine::SyncEngine;

use std::path::{Path, PathBuf};

pub fn map_src_to_dst(src_root: &Path, dst_root: &Path, src_path: &Path) -> Option<PathBuf> {
    src_path
        .strip_prefix(src_root)
        .ok()
        .map(|rel| dst_root.join(rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn maps_src_to_dst_correctly() {
        let src = PathBuf::from("/src");
        let dst = PathBuf::from("/dst");

        let file = PathBuf::from("/src/a/b/file.txt");
        let mapped = map_src_to_dst(&src, &dst, &file).unwrap();

        assert_eq!(mapped, PathBuf::from("/dst/a/b/file.txt"));
    }

    #[test]
    fn returns_none_for_paths_outside_src() {
        let src = PathBuf::from("/src");
        let dst = PathBuf::from("/dst");

        let other = PathBuf::from("/other/file.txt");
        assert!(map_src_to_dst(&src, &dst, &other).is_none());
    }
}
