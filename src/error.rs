use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum TyposError {
    #[error("no typos.toml found in '{0}' or any parent directory")]
    ConfigNotFound(PathBuf),

    #[error("failed to parse typos.toml: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("no profiles defined in typos.toml")]
    NoProfiles,

    #[error("profile '{0}' not found in typos.toml")]
    ProfileNotFound(String),

    #[error("font file not found: {0}")]
    FontNotFound(PathBuf),

    #[error("unsupported font format: {0} (expected .ttf, .otf, or .woff2)")]
    UnsupportedFont(PathBuf),

    #[error("woff2 decompression failed: {0}")]
    Woff2Decompress(String),

    #[error("typst compile error: {0}")]
    Compile(String),

    #[error("PDF export failed: {0}")]
    PdfExport(String),

    #[error("template file not found: {0}")]
    TemplateNotFound(PathBuf),

    #[error("{0} conversion(s) failed")]
    BatchFailed(usize),
}

pub(crate) type Result<T> = std::result::Result<T, TyposError>;
