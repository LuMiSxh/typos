use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "typos",
    version,
    about = "Self-contained Markdown to branded PDF converter",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Convert a single Markdown file to PDF
    Convert {
        /// Path to the Markdown file
        file: PathBuf,

        /// Profile name(s), comma-separated or "all" (e.g. "luca,hzd" or "all")
        #[arg(long = "profile", value_delimiter = ',')]
        profiles: Vec<String>,

        /// Override output path (only valid for a single profile)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert all Markdown files in a directory
    Batch {
        /// Directory to search for .md files (recursive)
        dir: PathBuf,

        /// Profile name(s), comma-separated or "all"
        #[arg(long = "profile", value_delimiter = ',')]
        profiles: Vec<String>,

        /// Override output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List available profiles from the nearest typos.toml
    List,

    /// Create a sample typos.toml in the current directory
    Init,
}
