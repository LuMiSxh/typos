//! Single source of truth for every default value typos uses.
//!
//! All defaults are stored as `&str` so they can be referenced from the config
//! resolver, the template variable block, the sample `typos.toml`, and the
//! docs without duplication. Default values that themselves want to track
//! another field use `$section.field` reference syntax (resolved post-merge).

// ── Identity ────────────────────────────────────────────────────────────────
pub(crate) const AUTHOR: &str = "";
pub(crate) const INSTITUTE: &str = "";
pub(crate) const EMAIL: &str = "";

// ── Colors ──────────────────────────────────────────────────────────────────
pub(crate) const COLOR_PRIMARY: &str = "#000000";
pub(crate) const COLOR_TEXT: &str = "#000000";
pub(crate) const COLOR_HEADING: &str = "$colors.text";
pub(crate) const COLOR_LINK: &str = "$colors.primary";
pub(crate) const COLOR_RULE: &str = "$colors.primary";
pub(crate) const COLOR_HEADER_LABEL: &str = "$colors.text";
pub(crate) const COLOR_CODE_FILL: &str = "#f6f6f6";
pub(crate) const COLOR_CODE_BORDER: &str = "#dcdcdc";
pub(crate) const COLOR_CODE_INLINE_FILL: &str = "#f0f0f0";
pub(crate) const COLOR_QUOTE_FILL: &str = "#fafafa";
pub(crate) const COLOR_QUOTE_BORDER: &str = "$colors.primary";
pub(crate) const COLOR_TABLE_STROKE: &str = "#cccccc";
pub(crate) const COLOR_TABLE_ALT_FILL: &str = "#fafafa";

// ── Sizes / spacing ─────────────────────────────────────────────────────────
pub(crate) const SIZE_BODY: &str = "11pt";
pub(crate) const SIZE_CODE: &str = "9.5pt";
pub(crate) const SIZE_TOP_MARGIN: &str = "3cm";
pub(crate) const SIZE_SIDE_MARGIN: &str = "2.5cm";
pub(crate) const SIZE_BOTTOM_MARGIN: &str = "2.5cm";
pub(crate) const SIZE_HEAD_HEIGHT: &str = "1.3cm";
pub(crate) const SIZE_LOGO_HEIGHT: &str = "1cm";
pub(crate) const SIZE_PAR_LEADING: &str = "0.65em";
pub(crate) const SIZE_PAR_SPACING: &str = "0.9em";
pub(crate) const SIZE_LIST_INDENT: &str = "1em";
pub(crate) const SIZE_LIST_SPACING: &str = "0.8em";
pub(crate) const SIZE_HEADING_ABOVE: &str = "1.4em";
pub(crate) const SIZE_HEADING_BELOW: &str = "0.6em";
pub(crate) const SIZE_H1: &str = "17pt";
pub(crate) const SIZE_H2: &str = "14pt";
pub(crate) const SIZE_H3: &str = "12pt";
pub(crate) const SIZE_H4: &str = "11pt";

// ── Fonts ───────────────────────────────────────────────────────────────────
pub(crate) const FONT_MAIN: &str = "Libertinus Serif";
pub(crate) const FONT_MONO: &str = "DejaVu Sans Mono";

// ── Layout ──────────────────────────────────────────────────────────────────
pub(crate) const HEADER_TEXT: &str = "";
