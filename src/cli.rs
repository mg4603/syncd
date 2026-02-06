use crate::ignore::IgnoreMatcher;
use crate::sync::SyncEngine;
use crate::watch::WatchEngine;
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "syncd", version, about = "Local-first folder sync tool")]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init { src: PathBuf, dst: PathBuf },
    Watch { src: PathBuf, dst: PathBuf },
}

pub fn run(args: Args) -> Result<()> {
    match args.cmd {
        Command::Init { src, dst } => {
            let (src, dst) = validate_paths(&src, &dst)?;
            println!("syncd init");
            println!("  src: {}", src.display());
            println!("  dst: {}", dst.display());
            print!("  status: syncing...");
            io::stdout().flush().unwrap();

            let ignore = IgnoreMatcher::new(&src);
            let sync = SyncEngine::new(&src, &dst, &ignore);

            sync.initial_sync()?;

            println!("\r  status: done");
            Ok(())
        }
        Command::Watch { src, dst } => {
            let (src, dst) = validate_paths(&src, &dst)?;

            println!("syncd watch");

            let ignore = IgnoreMatcher::new(&src);
            let sync = SyncEngine::new(&src, &dst, &ignore);
            let watch = WatchEngine::new(&src, &dst, &ignore);

            sync.initial_sync()?;
            println!("Initial sync done.");

            watch.watch_loop()?;

            Ok(())
        }
    }
}

fn validate_paths(src: &Path, dst: &Path) -> Result<(PathBuf, PathBuf)> {
    if !src.exists() {
        bail!("Source path does not exist: {}", src.display());
    }
    if !src.is_dir() {
        bail!("Source is not a directory: {}", src.display());
    }

    if !dst.exists() {
        std::fs::create_dir_all(dst).with_context(|| {
            format!("Failed to create destination directory: {}", dst.display())
        })?;
    }

    if !dst.is_dir() {
        bail!("Destination is not a directory: {}", dst.display());
    }

    let src_abs = std::fs::canonicalize(src)
        .with_context(|| format!("Failed to resolve source path: {}", src.display()))?;

    let dst_abs = std::fs::canonicalize(dst)
        .with_context(|| format!("Failed to resolve destination patth: {}", dst.display()))?;

    if dst_abs.starts_with(&src_abs) {
        bail!(
            "Destination cannot be inside source.\n  src: {}\n  dst:{}",
            src_abs.display(),
            dst_abs.display()
        );
    }

    Ok((src_abs, dst_abs))
}
