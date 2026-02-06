use crate::ignore::IgnoreMatcher;
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

pub struct SyncEngine<'a> {
    src: &'a Path,
    dst: &'a Path,
    ignore: &'a IgnoreMatcher,
}

impl<'a> SyncEngine<'a> {
    pub fn new(src: &'a Path, dst: &'a Path, ignore: &'a IgnoreMatcher) -> Self {
        Self { src, dst, ignore }
    }

    pub fn initial_sync(&self) -> Result<()> {
        let copied = AtomicUsize::new(0);
        let skipped = AtomicUsize::new(0);

        for entry in WalkDir::new(self.src).follow_links(false) {
            let entry = entry.context("WalkDir failed")?;
            let path = entry.path();

            if path == self.src || self.ignore.is_ignored(path) {
                continue;
            }

            let Some(dst_path) = crate::sync::map_src_to_dst(self.src, self.dst, path) else {
                continue;
            };

            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dst_path).with_context(|| {
                    format!("Failed to create directory: {}", dst_path.display())
                })?;
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
}
