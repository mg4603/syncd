use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

pub fn initial_sync(src: &Path, dst: &Path) -> Result<()> {
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
            crate::util::atomic_copy_file(path, &dst_path)?;
        }
    }
    Ok(())
}
