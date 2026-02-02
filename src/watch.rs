use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
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
                println!("fs event: {:?}", event);
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
