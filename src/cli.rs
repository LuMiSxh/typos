use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "typos",
    version,
    about = "Self-contained Markdown/Typst to branded PDF converter",
    long_about = None,
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Convert a single Markdown or Typst file to PDF
    Convert {
        /// Path to the .md or .typ source file
        file: PathBuf,

        /// Profile name(s), comma-separated or "all" (e.g. "luca,hzd" or "all")
        #[arg(long = "profile", value_delimiter = ',')]
        profiles: Vec<String>,

        /// Override output path (only valid for a single profile)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Open the resulting PDF after conversion
        #[arg(long)]
        open: bool,
    },

    /// Convert all .md/.typ files in a directory (recursive, parallel)
    Batch {
        /// Directory to search
        dir: PathBuf,

        /// Profile name(s), comma-separated or "all"
        #[arg(long = "profile", value_delimiter = ',')]
        profiles: Vec<String>,

        /// Override output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Watch a file or directory and re-convert on every change
    Watch {
        /// File or directory to watch
        path: PathBuf,

        /// Profile name(s), comma-separated or "all"
        #[arg(long = "profile", value_delimiter = ',')]
        profiles: Vec<String>,

        /// Override output path/directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List available profiles from the nearest typos.toml
    List,

    /// Create a sample typos.toml in the current directory
    Init,
}
