use std::path::Path;
use crate::config::FontSpec;
use crate::error::{Result, TyposError};

pub(crate) enum ResolvedFont {
    /// System font resolved by name via Typst's font searcher
    SystemName,
    /// File font: raw bytes (TTF or OTF)
    Bytes(Vec<u8>),
}

pub(crate) fn resolve(spec: &FontSpec, config_dir: &Path) -> Result<ResolvedFont> {
    match spec {
        FontSpec::Name(_) => Ok(ResolvedFont::SystemName),
        FontSpec::Path { path } => {
            let full_path = if std::path::Path::new(path).is_absolute() {
                std::path::PathBuf::from(path)
            } else {
                config_dir.join(path)
            };
            if !full_path.exists() {
                return Err(TyposError::FontNotFound(full_path));
            }
            let ext = full_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            match ext.as_str() {
                "ttf" | "otf" => {
                    let bytes = std::fs::read(&full_path)?;
                    Ok(ResolvedFont::Bytes(bytes))
                }
                "woff2" => {
                    let bytes = std::fs::read(&full_path)?;
                    let ttf = woff2_to_ttf(&bytes)
                        .map_err(TyposError::Woff2Decompress)?;
                    Ok(ResolvedFont::Bytes(ttf))
                }
                _ => Err(TyposError::UnsupportedFont(full_path)),
            }
        }
    }
}

/// Convert woff2 bytes to TTF bytes.
///
/// woff2 uses a "transformed tables" format that requires full SFNT reconstruction
/// after brotli decompression — it cannot be decoded with simple decompression.
/// This function provides a clear error message directing users to the proper tool.
fn woff2_to_ttf(woff2_bytes: &[u8]) -> std::result::Result<Vec<u8>, String> {
    // Check woff2 magic: "wOF2"
    if woff2_bytes.len() < 4 || &woff2_bytes[0..4] != b"wOF2" {
        return Err("not a valid woff2 file (missing wOF2 magic)".to_string());
    }

    // Full woff2 → TTF conversion requires SFNT table reconstruction after
    // brotli decompression — it is not a simple stream decode. Please convert
    // the font to TTF first using fonttools:
    //
    //   pip install fonttools brotli
    //   python -m fontTools.ttLib.woff2 decompress your-font.woff2
    //
    // This produces a .ttf file you can reference directly in typos.toml.
    Err(
        "woff2 fonts must be converted to TTF before use with typos. \
         Run: pip install fonttools brotli && \
         python -m fontTools.ttLib.woff2 decompress your-font.woff2"
            .to_string(),
    )
}
