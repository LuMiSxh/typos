mod cli;
mod config;
mod error;
mod font;
mod markdown;
mod template;
mod world;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => println!("got command: {:?}", cmd),
        None => println!("no command — interactive mode coming"),
    }
}
