use crate::config::{FontSpec, ResolvedProfile};
use crate::error::{Result, TyposError};

const DEFAULT_TEMPLATE: &str = include_str!("../assets/default.typ");

/// Assemble the full Typst source document:
///   1. Variable injection block (#let typos-X = ...)
///   2. Template (embedded default or profile override)
///   3. Markdown content (already converted to Typst markup)
pub fn assemble(profile: &ResolvedProfile, content: &str) -> Result<String> {
    let template = load_template(profile)?;
    let vars = build_variable_block(profile);
    Ok(format!("{vars}\n{template}\n\n{content}"))
}

fn load_template(profile: &ResolvedProfile) -> Result<String> {
    match &profile.template {
        Some(path) => {
            if !path.exists() {
                return Err(TyposError::TemplateNotFound(path.clone()));
            }
            Ok(std::fs::read_to_string(path)?)
        }
        None => Ok(DEFAULT_TEMPLATE.to_string()),
    }
}

/// Generate the #let variable injection block from a resolved profile.
fn build_variable_block(p: &ResolvedProfile) -> String {
    let logo_path = p.logo
        .as_ref()
        .map(|l| escape_typst_string(&l.to_string_lossy()))
        .unwrap_or_default();

    let logo_height = sanitize_length(&p.logo_height);
    let main_font = font_name(p);
    let mono_font = mono_font_name(p);

    format!(
        r#"#let typos-primary = rgb("{primary}")
#let typos-text-color = rgb("{text_color}")
#let typos-author = "{author}"
#let typos-institute = "{institute}"
#let typos-email = "{email}"
#let typos-logo-path = "{logo_path}"
#let typos-logo-height = {logo_height}
#let typos-header-text = "{header_text}"
#let typos-header-text-color = rgb("{header_text_color}")
#let typos-main-font = "{main_font}"
#let typos-mono-font = "{mono_font}"
"#,
        primary = p.primary_color.trim_start_matches('#'),
        text_color = p.text_color.trim_start_matches('#'),
        author = escape_typst_string(&p.author),
        institute = escape_typst_string(&p.institute),
        email = escape_typst_string(&p.email),
        logo_path = logo_path,
        logo_height = logo_height,
        header_text = escape_typst_string(&p.header_text),
        header_text_color = p.header_text_color.trim_start_matches('#'),
        main_font = main_font,
        mono_font = mono_font,
    )
}

fn escape_typst_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Sanitize a CSS/LaTeX length string for safe use as a Typst literal.
/// Accepts only digits, dots, and ASCII letters (e.g. "1cm", "2.5cm", "10mm").
fn sanitize_length(s: &str) -> &str {
    let s = s.trim();
    if s.chars().all(|c| c.is_ascii_digit() || c == '.' || c.is_ascii_alphabetic()) {
        s
    } else {
        "1cm"
    }
}

/// Extract the font family name for main_font.
/// For Name variants, use the name directly.
/// For Path variants, the font bytes are loaded into the Typst world;
/// we pass the PostScript family name. Since we don't parse the font here,
/// fall back to "Arial" — the user should set main_font to the family name
/// if using a file font.
fn font_name(p: &ResolvedProfile) -> String {
    match &p.main_font {
        FontSpec::Name(name) => escape_typst_string(name),
        FontSpec::Path { .. } => "Arial".to_string(),
    }
}

fn mono_font_name(p: &ResolvedProfile) -> String {
    match &p.mono_font {
        FontSpec::Name(name) => escape_typst_string(name),
        FontSpec::Path { .. } => "Courier New".to_string(),
    }
}
