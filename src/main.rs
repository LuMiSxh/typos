mod cli;
mod config;
mod convert;
mod error;
mod font;
mod frontmatter;
mod interactive;
mod markdown;
mod output;
mod render;
mod template;
mod watch;
mod world;

use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    if let Err(e) = run() {
        use console::style;
        eprintln!("\n{} {}", style("Error:").red().bold(), e);
        for cause in e.chain().skip(1) {
            eprintln!("  {} {}", style("caused by:").dim(), cause);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        Some(Command::Init) => cmd_init(&cwd)?,
        Some(Command::List) => cmd_list(&cwd)?,
        Some(Command::Convert { file, profiles, output, open }) => {
            cmd_convert(&cwd, file, profiles, output, open)?
        }
        Some(Command::Batch { dir, profiles, output }) => {
            cmd_batch(&cwd, dir, profiles, output)?
        }
        Some(Command::Watch { path, profiles, output }) => {
            cmd_watch(&cwd, path, profiles, output)?
        }
        None => cmd_interactive(&cwd)?,
    }
    Ok(())
}

fn cmd_list(cwd: &std::path::Path) -> error::Result<()> {
    let (config_dir, cfg) = config::discover(cwd)?;
    let profiles = cfg.resolve(&config_dir);
    if profiles.is_empty() {
        output::warn("no profiles defined in typos.toml");
        return Ok(());
    }
    output::header("Profiles");
    for p in &profiles {
        output::info(&format!("{} — {}", p.name, p.display_name));
    }
    Ok(())
}

fn cmd_convert(
    cwd: &std::path::Path,
    file: PathBuf,
    profile_args: Vec<String>,
    output: Option<PathBuf>,
    open: bool,
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
    let mut produced: Vec<PathBuf> = Vec::new();

    for profile in &selected {
        let result = convert::convert_file(&file, profile, output.as_deref(), suffix);
        convert::print_result(&file, &profile.name, &result);
        let path = result?;
        produced.push(path);
    }

    if open {
        for pdf in &produced {
            if let Err(e) = opener::open(pdf) {
                output::warn(&format!("could not open {}: {}", output::short_path(pdf), e));
            }
        }
    }
    Ok(())
}

fn cmd_watch(
    cwd: &std::path::Path,
    path: PathBuf,
    profile_args: Vec<String>,
    output: Option<PathBuf>,
) -> error::Result<()> {
    let (config_dir, cfg) = config::discover(cwd)?;
    let all_profiles = cfg.resolve(&config_dir);
    if all_profiles.is_empty() {
        return Err(error::TyposError::NoProfiles);
    }
    let selected = resolve_profile_args(&profile_args, &all_profiles)?;
    watch::run(&path, &selected, output.as_deref())
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
    report_batch_results(results)
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
            report_batch_results(results)
        }
    }
}

fn cmd_init(cwd: &std::path::Path) -> error::Result<()> {
    let dest = cwd.join("typos.toml");
    if dest.exists() {
        output::warn("typos.toml already exists");
        return Ok(());
    }
    std::fs::write(&dest, SAMPLE_TOML)?;
    output::ok("typos.toml", "created");
    Ok(())
}

fn report_batch_results(results: convert::BatchResults) -> error::Result<()> {
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
# Defaults apply to every profile unless overridden.
# Fonts default to bundled Libertinus Serif + DejaVu Sans Mono — these always
# work even on a fresh machine. Override with any system font name or a path.
# main_font = "Libertinus Serif"
# mono_font = "DejaVu Sans Mono"
# output_dir = "output"     # write PDFs into this folder instead of next to the source file
# template = "custom.typ"   # override the built-in Typst template

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
