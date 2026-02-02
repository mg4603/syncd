use anyhow::{Context, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch_loop(src: &Path, dst: &Path) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, notify::Config::default())
        .context("Failed to create filesystem watcher")?;

    watcher
        .watch(src, RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch source directory: {}", src.display()))?;

    println!("watch mode active");
    println!("  src: {}", src.display());
    println!("  dst: {}", dst.display());

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                apply_event(src, dst, event.unwrap())?;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // idle tick; keeps loop interruptible later
            }
            Err(e) => {
                return Err(e).context("watch channel error");
            }
        }
    }
}

fn apply_event(src_root: &Path, dst_root: &Path, event: Event) -> Result<()> {
    for path in event.paths {
        let Some(dst_path) = crate::sync::map_src_to_dst(src_root, dst_root, &path) else {
            continue;
        };

        if path.is_dir() {
            handle_dir_event(&path, &dst_path)?;
        } else {
            handle_file_event(&path, &dst_path)?;
        }
    }

    Ok(())
}

fn handle_file_event(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        crate::util::atomic_copy_file(src, dst)?;
        println!("file synced: {}", dst.display());
    } else if dst.exists() {
        std::fs::remove_file(dst)
            .with_context(|| format!("Failed to remove file: {}", dst.display()))?;
        println!("file removed: {}", dst.display());
    }
    Ok(())
}

fn handle_dir_event(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        std::fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create directory: {}", dst.display()))?;
        println!("dir ensured: {}", dst.display());
    } else if dst.exists() {
        std::fs::remove_dir_all(dst)
            .with_context(|| format!("Failed to remove directory: {}", dst.display()))?;
        println!("dir removed: {}", dst.display());
    }
    Ok(())
}
