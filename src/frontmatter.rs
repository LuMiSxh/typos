use crate::config::Profile;

/// Strip a leading TOML front-matter block from `source`.
///
/// Supported fences (must be the very first line, optionally preceded by a
/// UTF-8 BOM): `+++ … +++` or `--- … ---`. The block parses as TOML with the
/// same nested-section schema as `[[profiles]]` (sans `name`/`extends`).
///
/// Returns `(parsed_overrides, remaining_body)`. Missing or malformed
/// front-matter yields a default (empty) `Profile` and the original source.
pub(crate) fn split(source: &str) -> (Profile, &str) {
    let src = source.strip_prefix('\u{feff}').unwrap_or(source);

    let (fence, rest) = if let Some(rest) = src.strip_prefix("+++") {
        ("+++", rest)
    } else if let Some(rest) = src.strip_prefix("---") {
        ("---", rest)
    } else {
        return (Profile::default(), source);
    };

    let rest = match rest.strip_prefix('\r').unwrap_or(rest).strip_prefix('\n') {
        Some(r) => r,
        None => return (Profile::default(), source),
    };

    let needle_lf = format!("\n{fence}");
    let Some(close_rel) = rest.find(&needle_lf) else {
        return (Profile::default(), source);
    };
    let toml_block = &rest[..close_rel];
    let after_close = &rest[close_rel + needle_lf.len()..];

    let body_start = match after_close.chars().next() {
        Some('\n') => &after_close[1..],
        Some('\r') if after_close.starts_with("\r\n") => &after_close[2..],
        None => "",
        _ => return (Profile::default(), source),
    };

    match toml::from_str::<Profile>(toml_block) {
        Ok(p) => (p, body_start),
        Err(_) => (Profile::default(), source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_passes_through() {
        let (p, body) = split("# Hello\n\nWorld");
        assert!(p.identity.author.is_none());
        assert_eq!(body, "# Hello\n\nWorld");
    }

    #[test]
    fn nested_section_parses() {
        let src = "+++\n[identity]\nauthor = \"Luca\"\n+++\n# Hi";
        let (p, body) = split(src);
        assert_eq!(p.identity.author.as_deref(), Some("Luca"));
        assert_eq!(body, "# Hi");
    }

    #[test]
    fn dotted_key_parses() {
        let src = "+++\ncolors.primary = \"#FF0000\"\n+++\nbody";
        let (p, _) = split(src);
        assert_eq!(p.colors.primary.as_deref(), Some("#FF0000"));
    }

    #[test]
    fn dash_fence_parses() {
        let src = "---\n[identity]\nauthor = \"Luca\"\n---\nbody";
        let (p, body) = split(src);
        assert_eq!(p.identity.author.as_deref(), Some("Luca"));
        assert_eq!(body, "body");
    }

    #[test]
    fn malformed_toml_is_ignored() {
        let src = "+++\nthis is not toml\n+++\nbody";
        let (p, body) = split(src);
        assert!(p.identity.author.is_none());
        assert_eq!(body, src);
    }

    #[test]
    fn custom_var_passes_through() {
        let src = "+++\n[vars]\ncourse = \"Robotik\"\n+++\nbody";
        let (p, _) = split(src);
        assert_eq!(p.vars["course"].as_str(), Some("Robotik"));
    }
}
