use crate::ignore::IgnoreMatcher;
use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub fn watch_loop(src: &Path, dst: &Path, ignore: &IgnoreMatcher) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())
        .context("Failed to create filesystem watcher")?;

    watcher
        .watch(src, RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch source directory: {}", src.display()))?;

    println!("watch mode active");
    println!("  src: {}", src.display());
    println!("  dst: {}", dst.display());

    let debounce_window = Duration::from_millis(200);
    let mut pending: HashSet<std::path::PathBuf> = HashSet::new();
    let mut last_event_at: Option<Instant> = None;

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                for path in event.unwrap().paths {
                    pending.insert(path);
                }
                last_event_at = Some(Instant::now());
            }

            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(last) = last_event_at
                    && last.elapsed() >= debounce_window
                    && !pending.is_empty()
                {
                    for path in pending.drain() {
                        if ignore.is_ignored(&path) {
                            continue;
                        }

                        let Some(dst_path) = crate::sync::map_src_to_dst(src, dst, &path) else {
                            continue;
                        };

                        if path.exists() {
                            handle_present(&path, &dst_path)?;
                        } else {
                            handle_removal(&dst_path)?;
                        }
                    }
                    last_event_at = None;
                }
            }
            Err(e) => {
                return Err(e).context("watch channel error");
            }
        }
    }
}
fn handle_present(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create directory: {}", dst.display()))?;
        println!("dir ensured: {}", dst.display());
    } else {
        crate::util::atomic_copy_file(src, dst)?;
        println!("file synced: {}", dst.display());
    }

    Ok(())
}

fn handle_removal(dst: &Path) -> Result<()> {
    if !dst.exists() {
        return Ok(());
    }

    if dst.is_dir() {
        std::fs::remove_dir_all(dst)
            .with_context(|| format!("Failed to remove directory: {}", dst.display()))?;
        println!("dir removed: {}", dst.display());
    } else {
        std::fs::remove_file(dst)
            .with_context(|| format!("Failed to remove file: {}", dst.display()))?;
        println!("file removed: {}", dst.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn removal_is_idempotent_for_missing_paths() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("does-not-exist");

        handle_removal(&p).unwrap();
    }

    #[test]
    fn removes_file_correctly() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("file.txt");

        fs::write(&p, "data").unwrap();
        handle_removal(&p).unwrap();

        assert!(!p.exists());
    }

    #[test]
    fn removes_directory_correctly() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("dir");

        fs::create_dir_all(&p).unwrap();
        handle_removal(&p).unwrap();

        assert!(!p.exists());
    }
}
