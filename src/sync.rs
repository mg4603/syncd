use crate::ignore::IgnoreMatcher;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

pub fn initial_sync(src: &Path, dst: &Path, ignore: &IgnoreMatcher) -> Result<()> {
    let copied = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry.context("WalkDir failed")?;
        let path = entry.path();

        if ignore.is_ignored(path) || path == src {
            continue;
        }

        let Some(dst_path) = map_src_to_dst(src, dst, path) else {
            continue;
        };

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dst_path)
                .with_context(|| format!("Failed to create directory: {}", dst_path.display()))?;
            continue;
        }

        if entry.file_type().is_file() {
            if dst_path.exists() {
                let src_hash = crate::util::hash_file_blake3(path)?;
                let dst_hash = crate::util::hash_file_blake3(&dst_path)?;

                if src_hash == dst_hash {
                    skipped.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
            }
            crate::util::atomic_copy_file(path, &dst_path)?;
            copied.fetch_add(1, Ordering::Relaxed);
        }
    }

    println!(
        "sync summary: copied={}, skipped={}",
        copied.load(Ordering::Relaxed),
        skipped.load(Ordering::Relaxed),
    );

    Ok(())
}

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
