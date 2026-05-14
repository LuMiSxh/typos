mod cli;
mod config;
mod convert;
mod error;
mod font;
mod interactive;
mod markdown;
mod render;
mod template;
mod world;

use std::path::PathBuf;
use clap::Parser;
use cli::{Cli, Command};
use console::style;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()
        .map_err(error::TyposError::Io)?;

    match cli.command {
        Some(Command::Init) => cmd_init(&cwd),
        Some(Command::List) => cmd_list(&cwd),
        Some(Command::Convert { file, profiles, output }) => {
            cmd_convert(&cwd, file, profiles, output)
        }
        Some(Command::Batch { dir, profiles, output }) => {
            cmd_batch(&cwd, dir, profiles, output)
        }
        None => cmd_interactive(&cwd),
    }
}

fn cmd_list(cwd: &std::path::Path) -> error::Result<()> {
    let (config_dir, cfg) = config::discover(cwd)?;
    let profiles = cfg.resolve(&config_dir);
    if profiles.is_empty() {
        println!("No profiles defined in typos.toml");
        return Ok(());
    }
    println!("Profiles:");
    for p in &profiles {
        println!("  {} — {}", style(&p.name).cyan(), p.display_name);
    }
    Ok(())
}

fn cmd_convert(
    cwd: &std::path::Path,
    file: PathBuf,
    profile_args: Vec<String>,
    output: Option<PathBuf>,
) -> error::Result<()> {
    let (config_dir, cfg) = config::discover(cwd)?;
    let all_profiles = cfg.resolve(&config_dir);
    if all_profiles.is_empty() {
        return Err(error::TyposError::NoProfiles);
    }

    let selected = resolve_profile_args(&profile_args, &all_profiles)?;

    if output.is_some() && selected.len() > 1 {
        return Err(error::TyposError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "--output cannot be used with multiple profiles; omit --output or select a single profile",
        )));
    }

    let suffix = selected.len() > 1;

    for profile in &selected {
        let result = convert::convert_file(&file, profile, output.as_deref(), suffix);
        convert::print_result(&file, &profile.name, &result);
        result?;
    }
    Ok(())
}

fn cmd_batch(
    cwd: &std::path::Path,
    dir: PathBuf,
    profile_args: Vec<String>,
    output: Option<PathBuf>,
) -> error::Result<()> {
    let (config_dir, cfg) = config::discover(cwd)?;
    let all_profiles = cfg.resolve(&config_dir);
    if all_profiles.is_empty() {
        return Err(error::TyposError::NoProfiles);
    }

    let selected = resolve_profile_args(&profile_args, &all_profiles)?;

    let results = convert::batch(&dir, &selected, output.as_deref());
    let error_count = results.iter()
        .flat_map(|(_, file_results)| file_results.iter())
        .filter(|(_, r)| r.is_err())
        .count();
    for (md_path, file_results) in &results {
        for (profile_name, result) in file_results {
            convert::print_result(md_path, profile_name, result);
        }
    }
    if error_count > 0 {
        return Err(error::TyposError::BatchFailed(error_count));
    }
    Ok(())
}

fn cmd_interactive(cwd: &std::path::Path) -> error::Result<()> {
    let (config_dir, cfg) = config::discover(cwd)?;
    let all_profiles = cfg.resolve(&config_dir);

    use interactive::GuidedAction;
    match interactive::guided_flow(&all_profiles)? {
        GuidedAction::List => cmd_list(cwd),
        GuidedAction::ConvertFile { path, profiles: names } => {
            let selected: Vec<config::ResolvedProfile> = names.iter()
                .filter_map(|n| all_profiles.iter().find(|p| &p.name == n).cloned())
                .collect();
            if selected.is_empty() {
                return Err(error::TyposError::NoProfiles);
            }
            let suffix = selected.len() > 1;
            for profile in &selected {
                let result = convert::convert_file(&path, profile, None, suffix);
                convert::print_result(&path, &profile.name, &result);
                result?;
            }
            Ok(())
        }
        GuidedAction::Batch { dir, profiles: names } => {
            let selected: Vec<config::ResolvedProfile> = names.iter()
                .filter_map(|n| all_profiles.iter().find(|p| &p.name == n).cloned())
                .collect();
            if selected.is_empty() {
                return Err(error::TyposError::NoProfiles);
            }
            let results = convert::batch(&dir, &selected, None);
            let error_count = results.iter()
                .flat_map(|(_, file_results)| file_results.iter())
                .filter(|(_, r)| r.is_err())
                .count();
            for (md_path, file_results) in &results {
                for (profile_name, result) in file_results {
                    convert::print_result(md_path, profile_name, result);
                }
            }
            if error_count > 0 {
                return Err(error::TyposError::BatchFailed(error_count));
            }
            Ok(())
        }
    }
}

fn cmd_init(cwd: &std::path::Path) -> error::Result<()> {
    let dest = cwd.join("typos.toml");
    if dest.exists() {
        eprintln!("{} typos.toml already exists", style("warning:").yellow().bold());
        return Ok(());
    }
    std::fs::write(&dest, SAMPLE_TOML)?;
    println!("{} created typos.toml", style("✓").green());
    Ok(())
}

/// Resolve --profile args ("all" or comma-separated names) to profile list.
fn resolve_profile_args(
    args: &[String],
    all: &[config::ResolvedProfile],
) -> error::Result<Vec<config::ResolvedProfile>> {
    if args.is_empty() {
        // No --profile flag: use interactive picker or single auto-select
        if all.len() == 1 {
            return Ok(vec![all[0].clone()]);
        }
        let picked = interactive::pick_profiles(all)?;
        return Ok(picked.into_iter().cloned().collect());
    }
    if args.len() == 1 && args[0] == "all" {
        return Ok(all.to_vec());
    }
    let mut result = Vec::new();
    for name in args {
        let profile = all.iter()
            .find(|p| &p.name == name)
            .ok_or_else(|| error::TyposError::ProfileNotFound(name.clone()))?;
        result.push(profile.clone());
    }
    Ok(result)
}

const SAMPLE_TOML: &str = r##"[defaults]
output_dir = "output"
main_font = "Arial"
mono_font = "Consolas"
# template = "custom.typ"   # optional: override the built-in Typst template

[[profiles]]
name = "default"
display_name = "Default Profile"
primary_color = "#000000"
text_color = "#000000"
author = "Your Name"
institute = "Your Organisation"
email = "you@example.com"
# logo = "assets/logo.png"
logo_height = "1cm"
"##;
