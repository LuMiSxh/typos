mod cli;
mod config;
mod error;
mod font;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => println!("got command: {:?}", cmd),
        None => println!("no command — interactive mode coming"),
    }
}
