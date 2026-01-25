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
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create parent directory: {}", parent.display())
                })?;
            }

            std::fs::copy(path, &dst_path).with_context(|| {
                format!(
                    "Failed to copy file\n  src: {}\n  dst: {}",
                    path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}
