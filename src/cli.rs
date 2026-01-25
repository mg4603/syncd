use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "syncd", version, about = "Local-first folder sync tool")]
pub struct Args {}

pub fn run(_args: Args) -> Result<()> {
    Ok(())
}
