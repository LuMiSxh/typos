use std::collections::BTreeMap;

/// Strip a leading TOML front-matter block from `source`.
///
/// Supported delimiters (must be the very first line, optionally preceded
/// by a UTF-8 BOM):
/// ```text
/// +++
/// key = "value"
/// +++
/// ```
/// `---` is also accepted as a delimiter for familiarity with Hugo/Jekyll;
/// the body between fences is parsed as TOML regardless.
///
/// Returns `(overrides, remaining_body)`. If no front-matter is present,
/// returns an empty map and the original source unchanged.
pub(crate) fn split(source: &str) -> (BTreeMap<String, toml::Value>, &str) {
    let src = source.strip_prefix('\u{feff}').unwrap_or(source);

    let (fence, rest) = if let Some(rest) = src.strip_prefix("+++") {
        ("+++", rest)
    } else if let Some(rest) = src.strip_prefix("---") {
        ("---", rest)
    } else {
        return (BTreeMap::new(), source);
    };

    // Trailing whitespace + newline after the opening fence.
    let rest = match rest.strip_prefix('\r').unwrap_or(rest).strip_prefix('\n') {
        Some(r) => r,
        None => return (BTreeMap::new(), source),
    };

    // Find the closing fence on its own line.
    let needle_lf = format!("\n{fence}");
    let Some(close_rel) = rest.find(&needle_lf) else {
        return (BTreeMap::new(), source);
    };
    let toml_block = &rest[..close_rel];
    let after_close = &rest[close_rel + needle_lf.len()..];

    // Closing fence must be followed by newline or EOF.
    let body_start = match after_close.chars().next() {
        Some('\n') => &after_close[1..],
        Some('\r') if after_close.starts_with("\r\n") => &after_close[2..],
        None => "",
        _ => return (BTreeMap::new(), source),
    };

    match toml::from_str::<BTreeMap<String, toml::Value>>(toml_block) {
        Ok(map) => (map, body_start),
        Err(_) => (BTreeMap::new(), source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_passes_through() {
        let (vars, body) = split("# Hello\n\nWorld");
        assert!(vars.is_empty());
        assert_eq!(body, "# Hello\n\nWorld");
    }

    #[test]
    fn plus_fence_parses() {
        let src = "+++\nauthor = \"Luca\"\n+++\n# Hi";
        let (vars, body) = split(src);
        assert_eq!(vars["author"].as_str(), Some("Luca"));
        assert_eq!(body, "# Hi");
    }

    #[test]
    fn dash_fence_parses() {
        let src = "---\nauthor = \"Luca\"\n---\n# Hi";
        let (vars, body) = split(src);
        assert_eq!(vars["author"].as_str(), Some("Luca"));
        assert_eq!(body, "# Hi");
    }

    #[test]
    fn malformed_toml_is_ignored() {
        let src = "+++\nthis is not toml\n+++\nbody";
        let (vars, body) = split(src);
        assert!(vars.is_empty());
        assert_eq!(body, src);
    }

    #[test]
    fn custom_var_passes_through() {
        let src = "+++\ncourse = \"Robotik\"\n+++\nbody";
        let (vars, _) = split(src);
        assert_eq!(vars["course"].as_str(), Some("Robotik"));
    }
}
