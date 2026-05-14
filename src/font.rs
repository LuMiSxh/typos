use std::path::Path;
use crate::config::FontSpec;
use crate::error::{Result, TyposError};

pub enum ResolvedFont {
    /// System font: give this name to Typst's font searcher
    SystemName(String),
    /// File font: raw bytes (TTF or OTF)
    Bytes(Vec<u8>),
}

pub fn resolve(spec: &FontSpec, config_dir: &Path) -> Result<ResolvedFont> {
    match spec {
        FontSpec::Name(name) => Ok(ResolvedFont::SystemName(name.clone())),
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

/// Convert woff2 bytes to TTF bytes using brotli decompression.
///
/// woff2 = Brotli-compressed SFNT with a transformed table format.
/// We attempt basic decompression; for full production support the user
/// should convert woff2 → ttf with fonttools before use.
fn woff2_to_ttf(woff2_bytes: &[u8]) -> std::result::Result<Vec<u8>, String> {
    // Check woff2 magic: "wOF2"
    if woff2_bytes.len() < 4 || &woff2_bytes[0..4] != b"wOF2" {
        return Err("not a valid woff2 file (missing wOF2 magic)".to_string());
    }

    // woff2 total compressed size is at bytes 16..20 (big-endian u32)
    if woff2_bytes.len() < 48 {
        return Err("woff2 header too short".to_string());
    }

    let compressed_size = u32::from_be_bytes([
        woff2_bytes[16], woff2_bytes[17], woff2_bytes[18], woff2_bytes[19],
    ]) as usize;

    let _header_size = 48usize; // approximate; actual varies by table count
    let compressed_data_start = woff2_bytes.len().saturating_sub(compressed_size);

    let compressed = &woff2_bytes[compressed_data_start..];
    let mut decompressed = Vec::new();
    brotli::BrotliDecompress(&mut std::io::Cursor::new(compressed), &mut decompressed)
        .map_err(|e| format!("brotli decompression failed: {e}"))?;

    if decompressed.len() < 4 {
        return Err("decompressed data too short to be a valid font".to_string());
    }

    // Check for valid SFNT signature (TTF: 0x00010000 or 'true', OTF: 'OTTO')
    let sfnt_magic = &decompressed[0..4];
    if sfnt_magic == b"OTTO" || sfnt_magic == b"true"
        || sfnt_magic == &[0x00, 0x01, 0x00, 0x00]
    {
        Ok(decompressed)
    } else {
        Err(format!(
            "decompressed data has unexpected SFNT magic: {:02x?}. \
             Consider converting woff2 → ttf first using: \
             pip install fonttools && python -m fontTools.ttLib.woff2 decompress font.woff2",
            sfnt_magic
        ))
    }
}
