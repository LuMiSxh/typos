use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::defaults::*;
use crate::error::{Result, TyposError};

/// Font specification: either a system-installed font name or a path to a font file.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum FontSpec {
    Name(String),
    Path { path: String },
}

impl Default for FontSpec {
    fn default() -> Self {
        FontSpec::Name(FONT_MAIN.to_string())
    }
}

// ═══ TOML schema (input) ════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct IdentitySection {
    pub(crate) display_name: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) institute: Option<String>,
    pub(crate) email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ColorsSection {
    pub(crate) primary: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) heading: Option<String>,
    pub(crate) link: Option<String>,
    pub(crate) rule: Option<String>,
    pub(crate) header_label: Option<String>,
    pub(crate) code_fill: Option<String>,
    pub(crate) code_border: Option<String>,
    pub(crate) code_inline_fill: Option<String>,
    pub(crate) quote_fill: Option<String>,
    pub(crate) quote_border: Option<String>,
    pub(crate) table_stroke: Option<String>,
    pub(crate) table_alt_fill: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct SizesSection {
    pub(crate) body: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) top_margin: Option<String>,
    pub(crate) side_margin: Option<String>,
    pub(crate) bottom_margin: Option<String>,
    pub(crate) head_height: Option<String>,
    pub(crate) logo_height: Option<String>,
    pub(crate) par_leading: Option<String>,
    pub(crate) par_spacing: Option<String>,
    pub(crate) list_indent: Option<String>,
    pub(crate) list_spacing: Option<String>,
    pub(crate) heading_above: Option<String>,
    pub(crate) heading_below: Option<String>,
    pub(crate) h1: Option<String>,
    pub(crate) h2: Option<String>,
    pub(crate) h3: Option<String>,
    pub(crate) h4: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct FontsSection {
    pub(crate) main: Option<FontSpec>,
    pub(crate) mono: Option<FontSpec>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct LayoutSection {
    pub(crate) logo: Option<String>,
    pub(crate) header_text: Option<String>,
    pub(crate) template: Option<String>,
    pub(crate) output_dir: Option<String>,
}

/// `[defaults]` table — applied to every profile unless the profile overrides it.
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct Defaults {
    #[serde(default)]
    pub(crate) identity: IdentitySection,
    #[serde(default)]
    pub(crate) colors: ColorsSection,
    #[serde(default)]
    pub(crate) sizes: SizesSection,
    #[serde(default)]
    pub(crate) fonts: FontsSection,
    #[serde(default)]
    pub(crate) layout: LayoutSection,
    #[serde(default)]
    pub(crate) vars: BTreeMap<String, toml::Value>,
}

/// One `[[profiles]]` entry. Also used as the front-matter shape (where
/// `name`/`extends` are unused but the section fields override the profile).
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct Profile {
    #[serde(default)]
    pub(crate) name: String,
    pub(crate) extends: Option<String>,
    #[serde(default)]
    pub(crate) identity: IdentitySection,
    #[serde(default)]
    pub(crate) colors: ColorsSection,
    #[serde(default)]
    pub(crate) sizes: SizesSection,
    #[serde(default)]
    pub(crate) fonts: FontsSection,
    #[serde(default)]
    pub(crate) layout: LayoutSection,
    #[serde(default)]
    pub(crate) vars: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct TyposConfig {
    #[serde(default)]
    pub(crate) defaults: Defaults,
    #[serde(default, rename = "profiles")]
    pub(crate) profiles: Vec<Profile>,
}

// ═══ Resolved profile (output) ══════════════════════════════════════════════

#[derive(Debug, Clone)]
pub(crate) struct Identity {
    pub(crate) display_name: String,
    pub(crate) author: String,
    pub(crate) institute: String,
    pub(crate) email: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Colors {
    pub(crate) primary: String,
    pub(crate) text: String,
    pub(crate) heading: String,
    pub(crate) link: String,
    pub(crate) rule: String,
    pub(crate) header_label: String,
    pub(crate) code_fill: String,
    pub(crate) code_border: String,
    pub(crate) code_inline_fill: String,
    pub(crate) quote_fill: String,
    pub(crate) quote_border: String,
    pub(crate) table_stroke: String,
    pub(crate) table_alt_fill: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Sizes {
    pub(crate) body: String,
    pub(crate) code: String,
    pub(crate) top_margin: String,
    pub(crate) side_margin: String,
    pub(crate) bottom_margin: String,
    pub(crate) head_height: String,
    pub(crate) logo_height: String,
    pub(crate) par_leading: String,
    pub(crate) par_spacing: String,
    pub(crate) list_indent: String,
    pub(crate) list_spacing: String,
    pub(crate) heading_above: String,
    pub(crate) heading_below: String,
    pub(crate) h1: String,
    pub(crate) h2: String,
    pub(crate) h3: String,
    pub(crate) h4: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Fonts {
    pub(crate) main: FontSpec,
    pub(crate) mono: FontSpec,
}

#[derive(Debug, Clone)]
pub(crate) struct Layout {
    pub(crate) logo: Option<PathBuf>,
    pub(crate) header_text: String,
    pub(crate) template: Option<PathBuf>,
    /// `None` = write PDF next to the source file
    pub(crate) output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedProfile {
    pub(crate) name: String,
    pub(crate) identity: Identity,
    pub(crate) colors: Colors,
    pub(crate) sizes: Sizes,
    pub(crate) fonts: Fonts,
    pub(crate) layout: Layout,
    pub(crate) config_dir: PathBuf,
    /// Custom user-defined variables (injected as `typos-<key>`).
    pub(crate) vars: BTreeMap<String, toml::Value>,
}

// ═══ Discovery ══════════════════════════════════════════════════════════════

pub(crate) fn discover(start: &Path) -> Result<(PathBuf, TyposConfig)> {
    let mut dir = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        let candidate = dir.join("typos.toml");
        if candidate.is_file() {
            let content = std::fs::read_to_string(&candidate)?;
            let config: TyposConfig = toml::from_str(&content)?;
            return Ok((dir, config));
        }
        if !dir.pop() {
            break;
        }
    }
    Err(TyposError::ConfigNotFound(start.to_path_buf()))
}

// ═══ Resolution ═════════════════════════════════════════════════════════════

/// Resolve a single string field by walking the `extends` chain, then falling
/// back to defaults, then to the supplied constant.
macro_rules! resolve_str {
    ($chain:expr, $defaults:expr, $section:ident.$field:ident, $fallback:expr) => {{
        $chain
            .iter()
            .find_map(|p: &&Profile| p.$section.$field.clone())
            .or_else(|| $defaults.$section.$field.clone())
            .unwrap_or_else(|| $fallback.to_string())
    }};
}

/// Same pattern but for `Option<FontSpec>`.
macro_rules! resolve_font {
    ($chain:expr, $defaults:expr, $section:ident.$field:ident, $fallback:expr) => {{
        $chain
            .iter()
            .find_map(|p: &&Profile| p.$section.$field.clone())
            .or_else(|| $defaults.$section.$field.clone())
            .unwrap_or_else(|| FontSpec::Name($fallback.to_string()))
    }};
}

impl TyposConfig {
    pub(crate) fn resolve(&self, config_dir: &Path) -> Vec<ResolvedProfile> {
        let by_name: std::collections::HashMap<&str, &Profile> =
            self.profiles.iter().map(|p| (p.name.as_str(), p)).collect();
        self.profiles
            .iter()
            .map(|p| self.resolve_one(p, &by_name, config_dir))
            .collect()
    }

    fn resolve_one(
        &self,
        leaf: &Profile,
        by_name: &std::collections::HashMap<&str, &Profile>,
        config_dir: &Path,
    ) -> ResolvedProfile {
        // Build the extends chain leaf → root, stopping on cycles or missing parents.
        let mut chain: Vec<&Profile> = vec![leaf];
        let mut seen: std::collections::HashSet<&str> =
            std::iter::once(leaf.name.as_str()).collect();
        let mut cursor = leaf;
        while let Some(parent_name) = cursor.extends.as_deref() {
            let Some(parent) = by_name.get(parent_name) else { break };
            if !seen.insert(parent_name) {
                break;
            }
            chain.push(parent);
            cursor = parent;
        }
        let d = &self.defaults;

        let identity = Identity {
            display_name: resolve_str!(chain, d, identity.display_name, &leaf.name),
            author: resolve_str!(chain, d, identity.author, AUTHOR),
            institute: resolve_str!(chain, d, identity.institute, INSTITUTE),
            email: resolve_str!(chain, d, identity.email, EMAIL),
        };

        let colors = Colors {
            primary: resolve_str!(chain, d, colors.primary, COLOR_PRIMARY),
            text: resolve_str!(chain, d, colors.text, COLOR_TEXT),
            heading: resolve_str!(chain, d, colors.heading, COLOR_HEADING),
            link: resolve_str!(chain, d, colors.link, COLOR_LINK),
            rule: resolve_str!(chain, d, colors.rule, COLOR_RULE),
            header_label: resolve_str!(chain, d, colors.header_label, COLOR_HEADER_LABEL),
            code_fill: resolve_str!(chain, d, colors.code_fill, COLOR_CODE_FILL),
            code_border: resolve_str!(chain, d, colors.code_border, COLOR_CODE_BORDER),
            code_inline_fill: resolve_str!(
                chain, d, colors.code_inline_fill, COLOR_CODE_INLINE_FILL
            ),
            quote_fill: resolve_str!(chain, d, colors.quote_fill, COLOR_QUOTE_FILL),
            quote_border: resolve_str!(chain, d, colors.quote_border, COLOR_QUOTE_BORDER),
            table_stroke: resolve_str!(chain, d, colors.table_stroke, COLOR_TABLE_STROKE),
            table_alt_fill: resolve_str!(chain, d, colors.table_alt_fill, COLOR_TABLE_ALT_FILL),
        };

        let sizes = Sizes {
            body: resolve_str!(chain, d, sizes.body, SIZE_BODY),
            code: resolve_str!(chain, d, sizes.code, SIZE_CODE),
            top_margin: resolve_str!(chain, d, sizes.top_margin, SIZE_TOP_MARGIN),
            side_margin: resolve_str!(chain, d, sizes.side_margin, SIZE_SIDE_MARGIN),
            bottom_margin: resolve_str!(chain, d, sizes.bottom_margin, SIZE_BOTTOM_MARGIN),
            head_height: resolve_str!(chain, d, sizes.head_height, SIZE_HEAD_HEIGHT),
            logo_height: resolve_str!(chain, d, sizes.logo_height, SIZE_LOGO_HEIGHT),
            par_leading: resolve_str!(chain, d, sizes.par_leading, SIZE_PAR_LEADING),
            par_spacing: resolve_str!(chain, d, sizes.par_spacing, SIZE_PAR_SPACING),
            list_indent: resolve_str!(chain, d, sizes.list_indent, SIZE_LIST_INDENT),
            list_spacing: resolve_str!(chain, d, sizes.list_spacing, SIZE_LIST_SPACING),
            heading_above: resolve_str!(chain, d, sizes.heading_above, SIZE_HEADING_ABOVE),
            heading_below: resolve_str!(chain, d, sizes.heading_below, SIZE_HEADING_BELOW),
            h1: resolve_str!(chain, d, sizes.h1, SIZE_H1),
            h2: resolve_str!(chain, d, sizes.h2, SIZE_H2),
            h3: resolve_str!(chain, d, sizes.h3, SIZE_H3),
            h4: resolve_str!(chain, d, sizes.h4, SIZE_H4),
        };

        let fonts = Fonts {
            main: absolutise_font(resolve_font!(chain, d, fonts.main, FONT_MAIN), config_dir),
            mono: absolutise_font(resolve_font!(chain, d, fonts.mono, FONT_MONO), config_dir),
        };

        let layout = Layout {
            logo: chain
                .iter()
                .find_map(|p| p.layout.logo.clone())
                .or_else(|| d.layout.logo.clone())
                .map(|l| config_dir.join(l)),
            header_text: resolve_str!(chain, d, layout.header_text, HEADER_TEXT),
            template: chain
                .iter()
                .find_map(|p| p.layout.template.clone())
                .or_else(|| d.layout.template.clone())
                .map(|t| config_dir.join(t)),
            output_dir: chain
                .iter()
                .find_map(|p| p.layout.output_dir.clone())
                .or_else(|| d.layout.output_dir.clone())
                .map(|s| config_dir.join(s)),
        };

        // Merge custom vars: defaults → root → ... → leaf.
        let mut vars: BTreeMap<String, toml::Value> = d.vars.clone();
        for p in chain.iter().rev() {
            for (k, v) in &p.vars {
                vars.insert(k.clone(), v.clone());
            }
        }

        let mut profile = ResolvedProfile {
            name: leaf.name.clone(),
            identity,
            colors,
            sizes,
            fonts,
            layout,
            config_dir: config_dir.to_path_buf(),
            vars,
        };
        resolve_variable_refs(&mut profile);
        profile
    }
}

fn absolutise_font(spec: FontSpec, config_dir: &Path) -> FontSpec {
    match spec {
        FontSpec::Path { path } => FontSpec::Path {
            path: config_dir.join(&path).to_string_lossy().into_owned(),
        },
        other => other,
    }
}

// ═══ $section.field variable resolution ═════════════════════════════════════

/// Build a `(path, getter, setter)` table for every string-typed field, then
/// resolve `$section.field` references via fixed-point iteration with cycle
/// detection. Variables that fail to resolve keep their literal `$...` value.
fn resolve_variable_refs(profile: &mut ResolvedProfile) {
    // Flat snapshot of every resolvable path → current value.
    macro_rules! collect {
        ($($section:ident.$field:ident),* $(,)?) => {{
            let mut map = std::collections::HashMap::new();
            $(
                map.insert(
                    concat!(stringify!($section), ".", stringify!($field)).to_string(),
                    profile.$section.$field.clone(),
                );
            )*
            map
        }};
    }

    macro_rules! apply {
        ($map:expr, $($section:ident.$field:ident),* $(,)?) => {
            $(
                if let Some(v) = $map.get(concat!(stringify!($section), ".", stringify!($field))) {
                    profile.$section.$field = v.clone();
                }
            )*
        };
    }

    let mut map = collect!(
        identity.display_name, identity.author, identity.institute, identity.email,
        colors.primary, colors.text, colors.heading, colors.link, colors.rule,
        colors.header_label, colors.code_fill, colors.code_border, colors.code_inline_fill,
        colors.quote_fill, colors.quote_border, colors.table_stroke, colors.table_alt_fill,
        sizes.body, sizes.code, sizes.top_margin, sizes.side_margin, sizes.bottom_margin,
        sizes.head_height, sizes.logo_height, sizes.par_leading, sizes.par_spacing,
        sizes.list_indent, sizes.list_spacing, sizes.heading_above, sizes.heading_below,
        sizes.h1, sizes.h2, sizes.h3, sizes.h4,
        layout.header_text,
    );

    // Up to N passes of substitution. Each pass replaces every `$path` value
    // for which `path` resolves to a non-`$`-prefixed value. Stops early
    // when a pass changes nothing.
    for _ in 0..16 {
        let mut changed = false;
        let snapshot = map.clone();
        for (_, value) in map.iter_mut() {
            if let Some(target) = value.strip_prefix('$')
                && let Some(resolved) = snapshot.get(target)
                && !resolved.starts_with('$')
            {
                *value = resolved.clone();
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    apply!(map,
        identity.display_name, identity.author, identity.institute, identity.email,
        colors.primary, colors.text, colors.heading, colors.link, colors.rule,
        colors.header_label, colors.code_fill, colors.code_border, colors.code_inline_fill,
        colors.quote_fill, colors.quote_border, colors.table_stroke, colors.table_alt_fill,
        sizes.body, sizes.code, sizes.top_margin, sizes.side_margin, sizes.bottom_margin,
        sizes.head_height, sizes.logo_height, sizes.par_leading, sizes.par_spacing,
        sizes.list_indent, sizes.list_spacing, sizes.heading_above, sizes.heading_below,
        sizes.h1, sizes.h2, sizes.h3, sizes.h4,
        layout.header_text,
    );
}

// ═══ Front-matter overrides ═════════════════════════════════════════════════

impl ResolvedProfile {
    /// Apply a parsed front-matter Profile-shape on top of this resolved profile.
    /// Any field present in `overrides` replaces the matching resolved field.
    /// Custom vars merge over existing ones; `$var` references are re-resolved.
    pub(crate) fn with_overrides(mut self, overrides: &Profile) -> Self {
        macro_rules! merge_str {
            ($($section:ident.$field:ident),* $(,)?) => {
                $(
                    if let Some(v) = &overrides.$section.$field {
                        self.$section.$field = v.clone();
                    }
                )*
            };
        }
        merge_str!(
            identity.display_name, identity.author, identity.institute, identity.email,
            colors.primary, colors.text, colors.heading, colors.link, colors.rule,
            colors.header_label, colors.code_fill, colors.code_border, colors.code_inline_fill,
            colors.quote_fill, colors.quote_border, colors.table_stroke, colors.table_alt_fill,
            sizes.body, sizes.code, sizes.top_margin, sizes.side_margin, sizes.bottom_margin,
            sizes.head_height, sizes.logo_height, sizes.par_leading, sizes.par_spacing,
            sizes.list_indent, sizes.list_spacing, sizes.heading_above, sizes.heading_below,
            sizes.h1, sizes.h2, sizes.h3, sizes.h4,
            layout.header_text,
        );
        if let Some(spec) = &overrides.fonts.main {
            self.fonts.main = absolutise_font(spec.clone(), &self.config_dir);
        }
        if let Some(spec) = &overrides.fonts.mono {
            self.fonts.mono = absolutise_font(spec.clone(), &self.config_dir);
        }
        if let Some(logo) = &overrides.layout.logo {
            self.layout.logo = Some(self.config_dir.join(logo));
        }
        if let Some(t) = &overrides.layout.template {
            self.layout.template = Some(self.config_dir.join(t));
        }
        if let Some(o) = &overrides.layout.output_dir {
            self.layout.output_dir = Some(self.config_dir.join(o));
        }
        for (k, v) in &overrides.vars {
            self.vars.insert(k.clone(), v.clone());
        }
        resolve_variable_refs(&mut self);
        self
    }
}
