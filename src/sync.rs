use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

pub fn initial_sync(src: &Path, dst: &Path) -> Result<()> {
    let copied = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry.context("WalkDir failed")?;
        let path = entry.path();

        if path == src {
            continue;
        }

        let rel = path
            .strip_prefix(src)
            .context("Failed to compute relative path")?;
        let dst_path = dst.join(rel);

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
