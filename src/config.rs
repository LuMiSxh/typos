use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::error::{Result, TyposError};

/// Font specification: either a system-installed font name or a path to a font file.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FontSpec {
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
pub struct Defaults {
    pub output_dir: Option<String>,
    pub main_font: Option<FontSpec>,
    pub mono_font: Option<FontSpec>,
    pub template: Option<String>,
    pub top_margin: Option<String>,
    pub head_height: Option<String>,
}

/// One [[profiles]] entry in typos.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub name: String,
    pub display_name: Option<String>,
    pub primary_color: Option<String>,
    pub text_color: Option<String>,
    pub author: Option<String>,
    pub institute: Option<String>,
    pub email: Option<String>,
    pub logo: Option<String>,
    pub logo_height: Option<String>,
    pub header_text: Option<String>,
    pub header_text_color: Option<String>,
    pub main_font: Option<FontSpec>,
    pub mono_font: Option<FontSpec>,
    pub template: Option<String>,
    pub output_dir: Option<String>,
}

/// The full typos.toml structure.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TyposConfig {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default, rename = "profiles")]
    pub profiles: Vec<Profile>,
}

/// All fields resolved (defaults merged in, paths made absolute).
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub name: String,
    pub display_name: String,
    pub primary_color: String,
    pub text_color: String,
    pub author: String,
    pub institute: String,
    pub email: String,
    pub logo: Option<PathBuf>,
    pub logo_height: String,
    pub header_text: String,
    pub header_text_color: String,
    pub main_font: FontSpec,
    pub mono_font: FontSpec,
    pub template: Option<PathBuf>,
    pub output_dir: PathBuf,
    pub config_dir: PathBuf,
}

/// Walk up from `start` until a `typos.toml` is found.
/// Returns (config_dir, parsed config) or Err if not found.
pub fn discover(start: &Path) -> Result<(PathBuf, TyposConfig)> {
    let mut dir = start.to_path_buf();
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
    pub fn resolve(&self, config_dir: &Path) -> Vec<ResolvedProfile> {
        self.profiles.iter().map(|p| {
            let main_font = p.main_font.clone()
                .or_else(|| self.defaults.main_font.clone())
                .unwrap_or(FontSpec::Name("Arial".to_string()));
            let mono_font = p.mono_font.clone()
                .or_else(|| self.defaults.mono_font.clone())
                .unwrap_or(FontSpec::Name("Consolas".to_string()));
            let output_dir = p.output_dir.as_ref()
                .or(self.defaults.output_dir.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("output");
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
                header_text_color: p.header_text_color.clone().unwrap_or_else(|| "000000".to_string()),
                main_font,
                mono_font,
                template,
                output_dir: config_dir.join(output_dir),
                config_dir: config_dir.to_path_buf(),
            }
        }).collect()
    }
}
