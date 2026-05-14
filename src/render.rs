use std::collections::HashMap;
use typst::layout::PagedDocument;
use crate::config::ResolvedProfile;
use crate::error::{Result, TyposError};
use crate::font::{ResolvedFont, resolve as resolve_font};
use crate::world::{TyposWorld, collect_fonts};
use crate::{markdown, template};

/// Convert a Markdown string to PDF bytes using the given profile.
pub fn render(markdown_source: &str, profile: &ResolvedProfile) -> Result<Vec<u8>> {
    // 1. Convert Markdown → Typst markup
    let content = markdown::to_typst(markdown_source);

    // 2. Assemble full Typst source (variable block + template + content)
    let source = template::assemble(profile, &content)?;

    // 3. Collect extra font bytes (file/woff2 fonts from the profile)
    let mut extra_font_bytes: Vec<Vec<u8>> = Vec::new();
    for font_spec in [&profile.main_font, &profile.mono_font] {
        match resolve_font(font_spec, &profile.config_dir)? {
            ResolvedFont::SystemName => {}
            ResolvedFont::Bytes(bytes) => extra_font_bytes.push(bytes),
        }
    }

    // 4. Collect logo/image files for the world
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    if let Some(logo_path) = &profile.logo
        && logo_path.is_file() {
            let bytes = std::fs::read(logo_path)?;
            let key = format!("/{}", logo_path.to_string_lossy());
            files.insert(key.clone(), bytes.clone());
            files.insert(logo_path.to_string_lossy().to_string(), bytes);
        }

    // 5. Build Typst world
    let fonts = collect_fonts(extra_font_bytes);
    let world = TyposWorld::new(source, fonts, files, profile.config_dir.clone());

    // 6. Compile — typst::compile returns Warned<SourceResult<D>>
    // .output is SourceResult<PagedDocument> = Result<PagedDocument, EcoVec<SourceDiagnostic>>
    let result = typst::compile::<PagedDocument>(&world);
    let document = result.output.map_err(|errors| {
        let msgs: Vec<_> = errors.iter().map(|e| e.message.to_string()).collect();
        TyposError::Compile(msgs.join("; "))
    })?;

    // 7. Export PDF — typst_pdf::pdf(&document, &PdfOptions) -> SourceResult<Vec<u8>>
    let pdf_bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|errors| {
            let msgs: Vec<_> = errors.iter().map(|e| e.message.to_string()).collect();
            TyposError::PdfExport(msgs.join("; "))
        })?;

    Ok(pdf_bytes)
}
