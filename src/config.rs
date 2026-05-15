use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::Deserialize;
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
        FontSpec::Name("Libertinus Serif".to_string())
    }
}

/// The [defaults] section — values applied to all profiles unless overridden.
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct Defaults {
    pub(crate) output_dir: Option<String>,
    pub(crate) main_font: Option<FontSpec>,
    pub(crate) mono_font: Option<FontSpec>,
    pub(crate) template: Option<String>,
    pub(crate) top_margin: Option<String>,
    pub(crate) head_height: Option<String>,
    #[serde(default)]
    pub(crate) vars: BTreeMap<String, toml::Value>,
}

/// One [[profiles]] entry in typos.toml.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) extends: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) primary_color: Option<String>,
    pub(crate) text_color: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) institute: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) logo: Option<String>,
    pub(crate) logo_height: Option<String>,
    pub(crate) header_text: Option<String>,
    pub(crate) header_text_color: Option<String>,
    pub(crate) main_font: Option<FontSpec>,
    pub(crate) mono_font: Option<FontSpec>,
    pub(crate) template: Option<String>,
    pub(crate) output_dir: Option<String>,
    pub(crate) top_margin: Option<String>,
    pub(crate) head_height: Option<String>,
    #[serde(default)]
    pub(crate) vars: BTreeMap<String, toml::Value>,
}

/// The full typos.toml structure.
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct TyposConfig {
    #[serde(default)]
    pub(crate) defaults: Defaults,
    #[serde(default, rename = "profiles")]
    pub(crate) profiles: Vec<Profile>,
}

/// All fields resolved (defaults merged in, paths made absolute).
#[derive(Debug, Clone)]
pub(crate) struct ResolvedProfile {
    pub(crate) name: String,
    pub(crate) display_name: String,
    pub(crate) primary_color: String,
    pub(crate) text_color: String,
    pub(crate) author: String,
    pub(crate) institute: String,
    pub(crate) email: String,
    pub(crate) logo: Option<PathBuf>,
    pub(crate) logo_height: String,
    pub(crate) header_text: String,
    pub(crate) header_text_color: String,
    pub(crate) main_font: FontSpec,
    pub(crate) mono_font: FontSpec,
    pub(crate) template: Option<PathBuf>,
    /// None = write PDF next to the source file; Some = explicit directory
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) config_dir: PathBuf,
    pub(crate) top_margin: String,
    pub(crate) head_height: String,
    /// Custom user-defined variables (injected as `typos-<key>` in the Typst source).
    pub(crate) vars: BTreeMap<String, toml::Value>,
}

fn absolutise_font(spec: FontSpec, config_dir: &Path) -> FontSpec {
    match spec {
        FontSpec::Path { path } => FontSpec::Path {
            path: config_dir.join(&path).to_string_lossy().into_owned(),
        },
        other => other,
    }
}

/// Walk up from `start` until a `typos.toml` is found.
/// Returns (config_dir, parsed config) or Err if not found.
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

impl TyposConfig {
    /// Merge defaults into each profile, resolve `extends` chains, and absolutise paths.
    pub(crate) fn resolve(&self, config_dir: &Path) -> Vec<ResolvedProfile> {
        let by_name: std::collections::HashMap<&str, &Profile> = self
            .profiles
            .iter()
            .map(|p| (p.name.as_str(), p))
            .collect();

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

        macro_rules! pick {
            ($field:ident) => {
                chain.iter().find_map(|p| p.$field.clone())
            };
        }

        let main_font = absolutise_font(
            pick!(main_font)
                .or_else(|| self.defaults.main_font.clone())
                .unwrap_or(FontSpec::Name("Arial".to_string())),
            config_dir,
        );
        let mono_font = absolutise_font(
            pick!(mono_font)
                .or_else(|| self.defaults.mono_font.clone())
                .unwrap_or(FontSpec::Name("DejaVu Sans Mono".to_string())),
            config_dir,
        );
        let output_dir = pick!(output_dir)
            .or_else(|| self.defaults.output_dir.clone())
            .map(|s| config_dir.join(s));
        let template = pick!(template)
            .or_else(|| self.defaults.template.clone())
            .map(|t| config_dir.join(t));

        // Merge custom vars: defaults → root → ... → leaf (later overrides earlier).
        let mut vars: BTreeMap<String, toml::Value> = self.defaults.vars.clone();
        for p in chain.iter().rev() {
            for (k, v) in &p.vars {
                vars.insert(k.clone(), v.clone());
            }
        }

        ResolvedProfile {
            name: leaf.name.clone(),
            display_name: pick!(display_name).unwrap_or_else(|| leaf.name.clone()),
            primary_color: pick!(primary_color).unwrap_or_else(|| "#000000".to_string()),
            text_color: pick!(text_color).unwrap_or_else(|| "#000000".to_string()),
            author: pick!(author).unwrap_or_default(),
            institute: pick!(institute).unwrap_or_default(),
            email: pick!(email).unwrap_or_default(),
            logo: pick!(logo).map(|l| config_dir.join(l)),
            logo_height: pick!(logo_height).unwrap_or_else(|| "1cm".to_string()),
            header_text: pick!(header_text).unwrap_or_default(),
            header_text_color: pick!(header_text_color).unwrap_or_else(|| "#000000".to_string()),
            main_font,
            mono_font,
            template,
            output_dir,
            config_dir: config_dir.to_path_buf(),
            top_margin: pick!(top_margin)
                .or_else(|| self.defaults.top_margin.clone())
                .unwrap_or_else(|| "3cm".to_string()),
            head_height: pick!(head_height)
                .or_else(|| self.defaults.head_height.clone())
                .unwrap_or_else(|| "1.3cm".to_string()),
            vars,
        }
    }
}

impl ResolvedProfile {
    /// Apply a flat set of overrides (e.g. front-matter) on top of this profile.
    /// Known keys override the matching profile field; unknown keys go into `vars`.
    pub(crate) fn with_overrides(mut self, overrides: &BTreeMap<String, toml::Value>) -> Self {
        use toml::Value;
        fn as_str(v: &Value) -> Option<String> {
            match v {
                Value::String(s) => Some(s.clone()),
                Value::Integer(i) => Some(i.to_string()),
                Value::Float(f) => Some(f.to_string()),
                Value::Boolean(b) => Some(b.to_string()),
                _ => None,
            }
        }

        macro_rules! string_fields {
            ($($field:ident),* $(,)?) => {
                |k: &str, v: &Value, this: &mut ResolvedProfile| -> bool {
                    match k {
                        $(stringify!($field) => {
                            if let Some(s) = as_str(v) { this.$field = s; }
                            true
                        })*
                        _ => false,
                    }
                }
            };
        }
        let try_set = string_fields!(
            display_name, primary_color, text_color, author, institute, email,
            logo_height, header_text, header_text_color, top_margin, head_height,
        );

        for (k, v) in overrides {
            if try_set(k, v, &mut self) {
                continue;
            }
            if k == "logo" {
                if let Some(s) = as_str(v) {
                    self.logo = Some(self.config_dir.join(s));
                }
                continue;
            }
            self.vars.insert(k.clone(), v.clone());
        }
        self
    }
}
