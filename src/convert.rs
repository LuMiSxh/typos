use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::config::ResolvedProfile;
use crate::error::{Result, TyposError};
use crate::{frontmatter, markdown, output, render};

pub(crate) type BatchResults = Vec<(PathBuf, Vec<(String, Result<PathBuf>)>)>;

/// Convert a single source file (.md or .typ) with one profile.
/// Returns the path of the written PDF.
pub(crate) fn convert_file(
    src_path: &Path,
    profile: &ResolvedProfile,
    output_override: Option<&Path>,
    suffix_profile_name: bool,
) -> Result<PathBuf> {
    let source = std::fs::read_to_string(src_path)?;
    let (fm_overrides, body) = frontmatter::split(&source);
    let effective_profile = profile.clone().with_overrides(&fm_overrides);

    let typst_content = match source_kind(src_path) {
        SourceKind::Typst => body.to_string(),
        SourceKind::Markdown => markdown::to_typst(body),
    };

    let pdf_bytes = render::render(&typst_content, &effective_profile)?;

    let pdf_path = output_path(src_path, &effective_profile, output_override, suffix_profile_name)?;
    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&pdf_path, pdf_bytes)?;
    Ok(pdf_path)
}

/// Convert a single file with multiple profiles.
pub(crate) fn convert_file_multi(
    src_path: &Path,
    profiles: &[ResolvedProfile],
    output_override: Option<&Path>,
) -> Vec<(String, Result<PathBuf>)> {
    let suffix = profiles.len() > 1;
    profiles
        .iter()
        .map(|profile| {
            let result = convert_file(src_path, profile, output_override, suffix);
            (profile.name.clone(), result)
        })
        .collect()
}

/// Convert all .md/.typ files under `dir` with multiple profiles, in parallel.
pub(crate) fn batch(
    dir: &Path,
    profiles: &[ResolvedProfile],
    output_override: Option<&Path>,
) -> BatchResults {
    use rayon::prelude::*;

    let files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                output::warn(&format!("walkdir: {}", err));
                None
            }
        })
        .filter(|e| e.file_type().is_file() && is_supported_source(e.path()))
        .map(|e| e.path().to_owned())
        .collect();

    files
        .into_par_iter()
        .map(|path| {
            let results = convert_file_multi(&path, profiles, output_override);
            (path, results)
        })
        .collect()
}

/// Print a conversion result to the terminal.
pub(crate) fn print_result(src_path: &Path, profile_name: &str, result: &Result<PathBuf>) {
    let name = output::short_path(src_path);
    match result {
        Ok(pdf_path) => output::ok(&name, &format!("→ {}", output::short_path(pdf_path))),
        Err(e) => output::fail(&name, profile_name, &e.to_string()),
    }
}

#[derive(Copy, Clone)]
enum SourceKind {
    Markdown,
    Typst,
}

fn source_kind(path: &Path) -> SourceKind {
    match path.extension().and_then(|e| e.to_str()) {
        Some("typ") => SourceKind::Typst,
        _ => SourceKind::Markdown,
    }
}

pub(crate) fn is_supported_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md" | "typ")
    )
}

fn output_path(
    src_path: &Path,
    profile: &ResolvedProfile,
    output_override: Option<&Path>,
    suffix_profile_name: bool,
) -> Result<PathBuf> {
    if let Some(out) = output_override {
        return Ok(out.to_path_buf());
    }

    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            TyposError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "input file has no valid UTF-8 stem",
            ))
        })?;

    let filename = if suffix_profile_name {
        format!("{}_{}.pdf", stem, profile.name)
    } else {
        format!("{}.pdf", stem)
    };

    let dir = profile
        .layout
        .output_dir
        .as_deref()
        .unwrap_or_else(|| src_path.parent().unwrap_or(Path::new(".")));
    Ok(dir.join(&filename))
}
