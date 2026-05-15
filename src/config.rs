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
        FontSpec::Name("Arial".to_string())
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
}

/// One [[profiles]] entry in typos.toml.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Profile {
    pub(crate) name: String,
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
    /// Merge defaults into each profile and resolve all paths relative to config_dir.
    pub(crate) fn resolve(&self, config_dir: &Path) -> Vec<ResolvedProfile> {
        self.profiles.iter().map(|p| {
            let main_font = absolutise_font(
                p.main_font.clone()
                    .or_else(|| self.defaults.main_font.clone())
                    .unwrap_or(FontSpec::Name("Arial".to_string())),
                config_dir,
            );
            let mono_font = absolutise_font(
                p.mono_font.clone()
                    .or_else(|| self.defaults.mono_font.clone())
                    .unwrap_or(FontSpec::Name("Consolas".to_string())),
                config_dir,
            );
            let output_dir = p.output_dir.as_ref()
                .or(self.defaults.output_dir.as_ref())
                .map(|s| config_dir.join(s));
            let template = p.template.as_ref()
                .or(self.defaults.template.as_ref())
                .map(|t| config_dir.join(t));

            ResolvedProfile {
                name: p.name.clone(),
                display_name: p.display_name.clone().unwrap_or_else(|| p.name.clone()),
                primary_color: p.primary_color.clone().unwrap_or_else(|| "#000000".to_string()),
                text_color: p.text_color.clone().unwrap_or_else(|| "#000000".to_string()),
                author: p.author.clone().unwrap_or_default(),
                institute: p.institute.clone().unwrap_or_default(),
                email: p.email.clone().unwrap_or_default(),
                logo: p.logo.as_ref().map(|l| config_dir.join(l)),
                logo_height: p.logo_height.clone().unwrap_or_else(|| "1cm".to_string()),
                header_text: p.header_text.clone().unwrap_or_default(),
                header_text_color: p.header_text_color.clone().unwrap_or_else(|| "#000000".to_string()),
                main_font,
                mono_font,
                template,
                output_dir,
                config_dir: config_dir.to_path_buf(),
                top_margin: p.top_margin.clone()
                    .or_else(|| self.defaults.top_margin.clone())
                    .unwrap_or_else(|| "3cm".to_string()),
                head_height: p.head_height.clone()
                    .or_else(|| self.defaults.head_height.clone())
                    .unwrap_or_else(|| "1.3cm".to_string()),
            }
        }).collect()
    }
}
