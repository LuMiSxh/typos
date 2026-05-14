use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use console::style;
use crate::config::ResolvedProfile;
use crate::error::{Result, TyposError};
use crate::render;

type BatchResults = Vec<(PathBuf, Vec<(String, Result<PathBuf>)>)>;

/// Convert a single Markdown file with one profile.
/// Returns the path of the written PDF.
pub fn convert_file(
    md_path: &Path,
    profile: &ResolvedProfile,
    output_override: Option<&Path>,
    suffix_profile_name: bool,
) -> Result<PathBuf> {
    let source = std::fs::read_to_string(md_path)?;
    let pdf_bytes = render::render(&source, profile)?;

    let pdf_path = output_path(md_path, profile, output_override, suffix_profile_name)?;
    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&pdf_path, pdf_bytes)?;
    Ok(pdf_path)
}

/// Convert a single file with multiple profiles.
pub fn convert_file_multi(
    md_path: &Path,
    profiles: &[ResolvedProfile],
    output_override: Option<&Path>,
) -> Vec<(String, Result<PathBuf>)> {
    let suffix = profiles.len() > 1;
    profiles.iter().map(|profile| {
        let result = convert_file(md_path, profile, output_override, suffix);
        (profile.name.clone(), result)
    }).collect()
}

/// Convert all .md files under `dir` with multiple profiles.
pub fn batch(
    dir: &Path,
    profiles: &[ResolvedProfile],
    output_override: Option<&Path>,
) -> BatchResults {
    let md_files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!("  {} walkdir: {}", style("!").yellow(), err);
                None
            }
        })
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|x| x.to_str()) == Some("md")
        })
        .map(|e| e.path().to_owned())
        .collect();

    md_files.into_iter().map(|md_path| {
        let results = convert_file_multi(&md_path, profiles, output_override);
        (md_path, results)
    }).collect()
}

/// Print a conversion result to the terminal.
pub fn print_result(md_path: &Path, profile_name: &str, result: &Result<PathBuf>) {
    match result {
        Ok(pdf_path) => {
            println!(
                "  {} {} → {}",
                style("✓").green(),
                style(md_path.display()).dim(),
                style(pdf_path.display()).cyan(),
            );
        }
        Err(e) => {
            eprintln!(
                "  {} {} [{}]: {}",
                style("✗").red(),
                style(md_path.display()).dim(),
                style(profile_name).yellow(),
                style(e).red(),
            );
        }
    }
}

fn output_path(
    md_path: &Path,
    profile: &ResolvedProfile,
    output_override: Option<&Path>,
    suffix_profile_name: bool,
) -> Result<PathBuf> {
    if let Some(out) = output_override {
        return Ok(out.to_path_buf());
    }

    let stem = md_path.file_stem()
        .ok_or_else(|| TyposError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "input file has no stem",
        )))?
        .to_str()
        .ok_or_else(|| TyposError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "filename is not valid UTF-8",
        )))?;

    let filename = if suffix_profile_name {
        format!("{}_{}.pdf", stem, profile.name)
    } else {
        format!("{}.pdf", stem)
    };

    let dir = profile.output_dir.as_deref()
        .unwrap_or_else(|| md_path.parent().unwrap_or(Path::new(".")));
    Ok(dir.join(&filename))
}
