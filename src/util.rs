use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, copy};
use std::path::{Path, PathBuf};

// Copy src -> dst using atomic replace:
// write to a temp file in dst directory, then rename into place.

pub fn atomic_copy_file(src: &Path, dst: &Path) -> Result<()> {
    let parent = dst
        .parent()
        .context("Destination file has no parent directory")?;

    std::fs::create_dir_all(parent)
        .with_context(|| format!("Failed to created parent directory: {}", parent.display()))?;

    let tmp_path = tmp_path_for(dst);

    let src_f = File::open(src)
        .with_context(|| format!("Failed to open source file: {}", src.display()))?;
    let mut reader = BufReader::new(src_f);

    let tmp_f = File::create(&tmp_path)
        .with_context(|| format!("Failed to created temp file: {}", tmp_path.display()))?;
    let mut writer = BufWriter::new(tmp_f);

    copy(&mut reader, &mut writer).with_context(|| {
        format!(
            "Failed to copy bytes\n  src: {}\n  tmp: {}",
            src.display(),
            tmp_path.display()
        )
    })?;

    writer.flush().context("Failed to flush temp file")?;

    if dst.exists() {
        std::fs::remove_file(dst)
            .with_context(|| format!("Failed to remove existing dst file: {}", dst.display()))?;
    }

    std::fs::rename(&tmp_path, dst).with_context(|| {
        format!(
            "Failed to rename temp into place\n  tmp: {}\n  dst: {}",
            tmp_path.display(),
            dst.display()
        )
    })?;
    Ok(())
}

fn tmp_path_for(dst: &Path) -> PathBuf {
    // Simple deterministic name for MVP:
    // <filename>.syncd.tmp
    //
    // To be made unique later

    let filename = dst
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());

    dst.with_file_name(format!("{filename}.syncd.tmp"))
}
