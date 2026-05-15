use std::collections::HashMap;
use typst::layout::PagedDocument;
use crate::config::ResolvedProfile;
use crate::error::{Result, TyposError};
use crate::font::{ResolvedFont, resolve as resolve_font};
use crate::world::{TyposWorld, collect_fonts};
use crate::template;

/// Compile a Typst content string (already converted from Markdown if applicable)
/// to PDF bytes using the given profile's branding and template.
pub(crate) fn render(typst_content: &str, profile: &ResolvedProfile) -> Result<Vec<u8>> {
    let source = template::assemble(profile, typst_content)?;

    let mut extra_font_bytes: Vec<Vec<u8>> = Vec::new();
    for font_spec in [&profile.main_font, &profile.mono_font] {
        match resolve_font(font_spec, &profile.config_dir)? {
            ResolvedFont::SystemName => {}
            ResolvedFont::Bytes(bytes) => extra_font_bytes.push(bytes),
        }
    }

    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    if let Some(logo_path) = &profile.logo
        && logo_path.is_file()
    {
        let bytes = std::fs::read(logo_path)?;
        files.insert(logo_path.to_string_lossy().to_string(), bytes);
    }

    let fonts = collect_fonts(extra_font_bytes);
    let world = TyposWorld::new(source, fonts, files, profile.config_dir.clone());
    compile_pdf(&world)
}

fn compile_pdf(world: &TyposWorld) -> Result<Vec<u8>> {
    let result = typst::compile::<PagedDocument>(world);
    let document = result.output.map_err(|errors| {
        let msgs: Vec<_> = errors
            .iter()
            .map(|e| {
                let mut s = e.message.to_string();
                for hint in &e.hints {
                    s.push_str(&format!(" (hint: {})", hint));
                }
                s
            })
            .collect();
        TyposError::Compile(msgs.join("; "))
    })?;

    typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|errors| {
        let msgs: Vec<_> = errors.iter().map(|e| e.message.to_string()).collect();
        TyposError::PdfExport(msgs.join("; "))
    })
}
