use crate::config::{FontSpec, ResolvedProfile};
use crate::error::{Result, TyposError};

const DEFAULT_TEMPLATE: &str = include_str!("../assets/default.typ");

/// Assemble the full Typst source document:
///   1. Variable injection block (#let typos-X = ...)
///   2. Template (embedded default or profile override)
///   3. Content (already converted to Typst markup)
pub(crate) fn assemble(profile: &ResolvedProfile, content: &str) -> Result<String> {
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

fn build_variable_block(p: &ResolvedProfile) -> String {
    let logo_path = p.logo
        .as_ref()
        .map(|l| escape_typst_string(&l.to_string_lossy()))
        .unwrap_or_default();

    let logo_height = sanitize_length(&p.logo_height);
    let top_margin = sanitize_length(&p.top_margin);
    let head_height = sanitize_length(&p.head_height);
    let main_font = font_name_from_spec(&p.main_font, "Libertinus Serif");
    let mono_font = font_name_from_spec(&p.mono_font, "DejaVu Sans Mono");

    let mut out = format!(
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
#let typos-top-margin = {top_margin}
#let typos-head-height = {head_height}
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
        top_margin = top_margin,
        head_height = head_height,
    );

    // Custom vars: each becomes `#let typos-<key> = <typst literal>`.
    for (k, v) in &p.vars {
        if !is_valid_var_name(k) {
            continue;
        }
        out.push_str(&format!("#let typos-{} = {}\n", k, toml_to_typst(v)));
    }
    out
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

/// Extract a Typst-safe font family name from a FontSpec.
/// For `Path` variants the font bytes are loaded into the world under their embedded
/// family name; since we don't parse the font file here we can't know that name, so
/// users should pair a path font with a `Name` to specify the family explicitly.
fn font_name_from_spec(spec: &FontSpec, fallback: &str) -> String {
    match spec {
        FontSpec::Name(name) => escape_typst_string(name),
        FontSpec::Path { .. } => fallback.to_string(),
    }
}

/// Variable names must be ascii lowercase/digits/dash/underscore, starting with a letter.
fn is_valid_var_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Render a TOML value as a Typst literal.
fn toml_to_typst(v: &toml::Value) -> String {
    use toml::Value;
    match v {
        Value::String(s) => format!("\"{}\"", escape_typst_string(s)),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(toml_to_typst).collect();
            format!("({})", parts.join(", "))
        }
        Value::Table(t) => {
            let parts: Vec<String> = t
                .iter()
                .filter(|(k, _)| is_valid_var_name(k))
                .map(|(k, v)| format!("{}: {}", k, toml_to_typst(v)))
                .collect();
            format!("({})", parts.join(", "))
        }
        Value::Datetime(d) => format!("\"{}\"", d),
    }
}
