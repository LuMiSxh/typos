use std::path::Path;
use crate::config::{FontSpec, ResolvedProfile};
use crate::error::{Result, TyposError};
use crate::defaults;

/// Portable (forward-slash, no drive letter) virtual path used to reference
/// the profile logo from the generated Typst source and to key the
/// preloaded logo bytes in `TyposWorld`. Embedding the logo's raw OS path
/// instead breaks on Windows: Typst's path resolution strips the drive
/// letter (or, on newer Typst, rejects backslashes outright), so the
/// `#image(...)` call and the preloaded file table would disagree on the
/// file's location.
pub(crate) fn logo_virtual_path(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => format!("/typos-logo.{ext}"),
        None => "/typos-logo".to_string(),
    }
}

const DEFAULT_TEMPLATE: &str = include_str!("../assets/default.typ");

/// Assemble the full Typst source document:
///   1. Variable injection block (every `typos-<section>-<field>` binding)
///   2. Template (embedded default or profile override)
///   3. Content (already Typst markup)
pub(crate) fn assemble(profile: &ResolvedProfile, content: &str) -> Result<String> {
    let template = load_template(profile)?;
    let vars = build_variable_block(profile);
    Ok(format!("{vars}\n{template}\n\n{content}"))
}

fn load_template(profile: &ResolvedProfile) -> Result<String> {
    match &profile.layout.template {
        Some(path) => {
            if !path.exists() {
                return Err(TyposError::TemplateNotFound(path.clone()));
            }
            Ok(std::fs::read_to_string(path)?)
        }
        None => Ok(DEFAULT_TEMPLATE.to_string()),
    }
}

/// Build the `#let typos-...` injection block. Each emitted binding is
/// `typos-<section>-<field>`, typed appropriately (colors as `rgb(...)`,
/// lengths as raw literals, strings quoted, fonts as quoted strings).
fn build_variable_block(p: &ResolvedProfile) -> String {
    let mut out = String::with_capacity(2048);

    // Strings — identity + free-form layout text.
    write_str(&mut out, "identity-display-name", &p.identity.display_name);
    write_str(&mut out, "identity-author", &p.identity.author);
    write_str(&mut out, "identity-institute", &p.identity.institute);
    write_str(&mut out, "identity-email", &p.identity.email);
    write_str(&mut out, "layout-header-text", &p.layout.header_text);
    write_str(
        &mut out,
        "layout-logo-path",
        &p.layout
            .logo
            .as_ref()
            .map(|l| logo_virtual_path(l))
            .unwrap_or_default(),
    );

    // Fonts (font family name as quoted string).
    write_str(&mut out, "fonts-main", &font_name(&p.fonts.main, defaults::FONT_MAIN));
    write_str(&mut out, "fonts-mono", &font_name(&p.fonts.mono, defaults::FONT_MONO));

    // Colors — hex strings become `rgb("...")`.
    write_color(&mut out, "colors-primary", &p.colors.primary);
    write_color(&mut out, "colors-text", &p.colors.text);
    write_color(&mut out, "colors-heading", &p.colors.heading);
    write_color(&mut out, "colors-link", &p.colors.link);
    write_color(&mut out, "colors-rule", &p.colors.rule);
    write_color(&mut out, "colors-header-label", &p.colors.header_label);
    write_color(&mut out, "colors-code-fill", &p.colors.code_fill);
    write_color(&mut out, "colors-code-border", &p.colors.code_border);
    write_color(&mut out, "colors-code-inline-fill", &p.colors.code_inline_fill);
    write_color(&mut out, "colors-quote-fill", &p.colors.quote_fill);
    write_color(&mut out, "colors-quote-border", &p.colors.quote_border);
    write_color(&mut out, "colors-table-stroke", &p.colors.table_stroke);
    write_color(&mut out, "colors-table-alt-fill", &p.colors.table_alt_fill);

    // Sizes / spacing — written as raw Typst length literals after sanitation.
    write_length(&mut out, "sizes-body", &p.sizes.body);
    write_length(&mut out, "sizes-code", &p.sizes.code);
    write_length(&mut out, "sizes-top-margin", &p.sizes.top_margin);
    write_length(&mut out, "sizes-side-margin", &p.sizes.side_margin);
    write_length(&mut out, "sizes-bottom-margin", &p.sizes.bottom_margin);
    write_length(&mut out, "sizes-head-height", &p.sizes.head_height);
    write_length(&mut out, "sizes-logo-height", &p.sizes.logo_height);
    write_length(&mut out, "sizes-par-leading", &p.sizes.par_leading);
    write_length(&mut out, "sizes-par-spacing", &p.sizes.par_spacing);
    write_length(&mut out, "sizes-list-indent", &p.sizes.list_indent);
    write_length(&mut out, "sizes-list-spacing", &p.sizes.list_spacing);
    write_length(&mut out, "sizes-heading-above", &p.sizes.heading_above);
    write_length(&mut out, "sizes-heading-below", &p.sizes.heading_below);
    write_length(&mut out, "sizes-h1", &p.sizes.h1);
    write_length(&mut out, "sizes-h2", &p.sizes.h2);
    write_length(&mut out, "sizes-h3", &p.sizes.h3);
    write_length(&mut out, "sizes-h4", &p.sizes.h4);

    // Custom vars become `typos-<key>` with TOML-aware literal rendering.
    for (k, v) in &p.vars {
        if is_valid_var_name(k) {
            out.push_str(&format!("#let typos-{} = {}\n", k, toml_to_typst(v)));
        }
    }

    out
}

// ── Writers ─────────────────────────────────────────────────────────────────

fn write_str(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!("#let typos-{} = \"{}\"\n", name, escape_typst_string(value)));
}

fn write_color(out: &mut String, name: &str, hex: &str) {
    let cleaned = hex.trim_start_matches('#');
    out.push_str(&format!("#let typos-{} = rgb(\"{}\")\n", name, cleaned));
}

fn write_length(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!("#let typos-{} = {}\n", name, sanitize_length(value)));
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn escape_typst_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Accept only digits, dots, and ASCII letters (e.g. "1cm", "0.65em", "10pt"),
/// fall back to a safe literal when the input contains anything else.
fn sanitize_length(s: &str) -> String {
    let t = s.trim();
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit() || c == '.' || c.is_ascii_alphabetic())
    {
        t.to_string()
    } else {
        "1cm".to_string()
    }
}

fn font_name(spec: &FontSpec, fallback: &str) -> String {
    match spec {
        FontSpec::Name(name) => escape_typst_string(name),
        FontSpec::Path { .. } => fallback.to_string(),
    }
}

fn is_valid_var_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

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
