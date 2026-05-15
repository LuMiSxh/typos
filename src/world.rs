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
        let offset_hours = offset.unwrap_or(0);
        let utc_offset = time::UtcOffset::from_hms(offset_hours.try_into().ok()?, 0, 0).ok()?;
        let now = time::OffsetDateTime::now_utc().to_offset(utc_offset);
        Datetime::from_ymd(now.year(), now.month() as u8, now.day())
    }
}

/// Cached bundled + system font scan (loaded once per process, shared across all renders).
/// `Font` is Arc-backed internally, so cloning the Vec is cheap.
static BASE_FONTS: std::sync::OnceLock<Vec<Font>> = std::sync::OnceLock::new();

fn load_base_fonts() -> Vec<Font> {
    let mut fonts = Vec::new();
    for data in typst_assets::fonts() {
        let bytes = Bytes::new(data.to_vec());
        for font in Font::iter(bytes) {
            fonts.push(font);
        }
    }
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
    fonts
}

/// Collect fonts: cached (bundled typst-assets + system) + extra bytes from profile.
pub(crate) fn collect_fonts(extra_font_bytes: Vec<Vec<u8>>) -> Vec<Font> {
    let mut fonts = BASE_FONTS.get_or_init(load_base_fonts).clone();
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
