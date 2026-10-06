use anyhow::Result;
use clap::{Parser, Subcommand};

mod modules;

use modules::gallery::assemble_gallery;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Spooky Maze maintenance tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assemble the ESPBrew snapshot gallery (deploy/site/index.html) from the
    /// per-board snapshot zips in deploy/site/zips/ and the optional WASM build
    /// in deploy/wasm/. Run from the repo root.
    Gallery {
        #[arg(long, short)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Gallery { verbose } => assemble_gallery(verbose),
    }
}
