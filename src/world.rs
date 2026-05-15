use std::collections::HashMap;
use std::path::PathBuf;

use typst::LibraryExt;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::Library;

pub(crate) struct TyposWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    files: HashMap<String, Bytes>,
    base_dir: PathBuf,
}

impl TyposWorld {
    pub(crate) fn new(
        source_text: String,
        fonts: Vec<Font>,
        files: HashMap<String, Vec<u8>>,
        base_dir: PathBuf,
    ) -> Self {
        let mut book = FontBook::new();
        for font in &fonts {
            book.push(font.info().clone());
        }
        let files: HashMap<String, Bytes> = files
            .into_iter()
            .map(|(k, v)| (k, Bytes::new(v)))
            .collect();
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        let source = Source::new(main_id, source_text);
        Self {
            source,
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(book),
            fonts,
            files,
            base_dir,
        }
    }
}

impl typst::World for TyposWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            return Ok(self.source.clone());
        }
        Err(FileError::NotFound(
            id.vpath().as_rooted_path().to_owned(),
        ))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let vpath = id.vpath().as_rooted_path();
        let key = vpath.to_string_lossy().to_string();
        if let Some(bytes) = self.files.get(&key) {
            return Ok(bytes.clone());
        }
        // Try reading from filesystem relative to base_dir
        let abs = self
            .base_dir
            .join(vpath.strip_prefix("/").unwrap_or(vpath));
        if abs.is_file() {
            return std::fs::read(&abs)
                .map(Bytes::new)
                .map_err(|_| FileError::NotFound(abs));
        }
        Err(FileError::NotFound(vpath.to_owned()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?;
        // Apply UTC offset in hours
        let secs = now.as_secs() as i64 + offset.unwrap_or(0) * 3600;
        let days = secs.div_euclid(86400) as i32;

        // Convert days since 1970-01-01 to a calendar date
        // Using the proleptic Gregorian calendar algorithm
        let (year, month, day) = days_since_epoch_to_ymd(days);
        Datetime::from_ymd(year, month, day)
    }
}

/// Convert days since the Unix epoch (1970-01-01) to a (year, month, day) tuple.
///
/// Uses the algorithm from <https://howardhinnant.github.io/date_algorithms.html>
/// (civil_from_days).
fn days_since_epoch_to_ymd(z: i32) -> (i32, u8, u8) {
    let z = z as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u8, d as u8)
}

/// Collect fonts: bundled typst-assets fonts + system fonts + extra bytes from profiles.
pub(crate) fn collect_fonts(extra_font_bytes: Vec<Vec<u8>>) -> Vec<Font> {
    let mut fonts = Vec::new();

    // Bundled Typst fonts (required for standard library)
    for data in typst_assets::fonts() {
        let bytes = Bytes::new(data.to_vec());
        for font in Font::iter(bytes) {
            fonts.push(font);
        }
    }

    // System fonts (recursively scan well-known directories)
    for dir in system_font_dirs() {
        for entry in walkdir::WalkDir::new(&dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "otc")
                && let Ok(data) = std::fs::read(path)
            {
                for font in Font::iter(Bytes::new(data)) {
                    fonts.push(font);
                }
            }
        }
    }

    // Extra (profile-specified file fonts)
    for raw in extra_font_bytes {
        for font in Font::iter(Bytes::new(raw)) {
            fonts.push(font);
        }
    }

    fonts
}

fn system_font_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "macos")]
    {
        dirs.push("/Library/Fonts".into());
        dirs.push("/System/Library/Fonts".into());
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(std::path::PathBuf::from(home).join("Library/Fonts"));
        }
    }
    #[cfg(target_os = "linux")]
    {
        dirs.push("/usr/share/fonts".into());
        dirs.push("/usr/local/share/fonts".into());
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(std::path::PathBuf::from(&home).join(".fonts"));
            dirs.push(std::path::PathBuf::from(home).join(".local/share/fonts"));
        }
    }
    #[cfg(windows)]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            dirs.push(std::path::PathBuf::from(windir).join("Fonts"));
        }
        if let Some(appdata) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(
                std::path::PathBuf::from(appdata).join("Microsoft/Windows/Fonts"),
            );
        }
    }
    dirs
}
