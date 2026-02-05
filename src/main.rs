mod cli;
mod ignore;
mod sync;
mod util;
mod watch;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    cli::run(args)
}
