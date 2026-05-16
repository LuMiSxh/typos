# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - 2026-05-16

### Added

- **Fully customizable theme**: every color, size, and spacing value in the built-in template is now an optional profile field. ~30 new keys grouped into sections (`identity`, `colors`, `sizes`, `fonts`, `layout`) — leave them out and the built-in default is used.
- **`$section.field` variable references** in any string value. Example: `heading = "$colors.text"` means "track the body text color". Derived defaults (`link → primary`, `rule → primary`, `quote_border → primary`, `heading → text`) use this mechanism, so changing one root color propagates automatically.
- **`src/defaults.rs`**: single source of truth for every default value. No more duplicate string literals scattered across resolver, template, sample TOML, and docs.

### Changed

- **TOML schema restructured into nested sections** (breaking change vs. 0.3.0):
  ```toml
  [[profiles]]
  name = "luca"
  [profiles.identity]
  author = "Luca Schmidt"
  [profiles.colors]
  primary = "#ED1B24"
  text    = "#000000"
  [profiles.sizes]
  top_margin = "3cm"
  ```
  Old flat keys (`primary_color`, `text_color`, `top_margin`, etc.) are gone — section paths replace them. Front-matter accepts the same nested shape (or dotted keys: `colors.primary = "..."`).
- **Template weights softened**: headings and `**strong**` text both render at `semibold` instead of `bold` for a more refined look on Libertinus Serif. Headings now use `text` color by default (overridable to any color including `$colors.primary` for branded headings).
- **Front-matter parser** now deserializes into the same `Profile` struct as `[[profiles]]`, so every theme field is overridable per-document.
- **`extends` semantics** updated to work at section/field granularity: missing fields walk the chain, mixed defaults compose cleanly.

## [0.3.0] - 2026-05-15

### Added

- **Typst file input**: `typos convert report.typ` now works — `.typ` source files are wrapped in the same branded template as Markdown, skipping only the Markdown→Typst conversion step. Batch picks up both `.md` and `.typ`.
- **Document front-matter**: any source file (`.md` or `.typ`) may begin with a TOML front-matter block (`+++ … +++` or `--- … ---`). Recognised keys override profile fields just for that file; unknown keys are exposed to the template as `typos-<key>`.
- **Profile inheritance via `extends`**: a profile can declare `extends = "<other_profile>"` to inherit every field; missing values walk the chain leaf → parent → grandparent. Cycles are detected and broken safely.
- **Custom template variables**: `[defaults.vars]` and `[profiles.X.vars]` tables expose arbitrary user values to templates as `typos-<key>` bindings. Supports strings, numbers, bools, arrays, and inline tables.
- **`watch` command**: `typos watch path --profile X` watches a file or directory and re-converts on every change, with per-path debouncing.
- **`--open` flag**: `typos convert ... --open` opens the resulting PDF after a successful conversion.

### Changed

- **Default fonts switched to bundled**: `main_font` defaults to `"Libertinus Serif"`, `mono_font` to `"DejaVu Sans Mono"` (both shipped inside the binary). Identical, predictable output on every machine — no more "font not found" surprises when system Arial/Consolas are missing.
- **Built-in template redesigned**: uniform bold-only heading weights with a cleaner size scale (17/14/12/11 pt), headings in body text color (safe for any brand palette), larger code blocks (9.5 pt), inline code at relative size (0.92 em), tighter and more even paragraph/list spacing, and softer table styling.
- **Batch conversion is now parallel** (rayon) and the bundled+system font scan is cached across the whole process — large batches are dramatically faster.
- **Date implementation**: replaced the hand-rolled proleptic-Gregorian calendar with the `time` crate. Same behavior, less code to maintain.
- **`output_path` simplified** — collapsed two `Io` boxing chains into one.

## [0.2.0] - 2026-05-15

### Added

- Typst math passthrough: inline `$...$` and display `$$...$$` in Markdown are now rendered as native Typst math instead of escaped literal text.

### Fixed

- Logo images no longer silently fail to load — the redundant double-slash virtual path key (`//absolute/path`) was removed, leaving only the correct key that Typst actually looks up.
- Interactive guided flow no longer silently falls back to the first profile when the user confirms with nothing selected; it now returns an error consistently with the non-interactive picker.
- Math compilation no longer fails with "no font could be found" — `typst-assets` is now built with the `fonts` feature so the bundled math font (NewCM Math) is actually loaded.
- Compile diagnostics now include Typst's hints alongside the message, making errors like unknown math symbols self-explanatory.

### Changed

- Batch result reporting extracted into a shared helper, eliminating duplicated logic between `convert` and interactive batch commands.
- Terminal output moved to a dedicated `output` module matching the style of the dots project: `✓`/`!!`/`!` symbols, bold names, dim messages, all routed through `console::Term` instead of raw `println!`/`eprintln!`.
- Error display now prints the full cause chain (`caused by:`) using `anyhow`; `run()` returns `anyhow::Result` while internal modules keep typed `TyposError` via `thiserror`.
- File paths in output trimmed to `parent/filename` instead of full absolute paths.
- All internal visibility changed from `pub` to `pub(crate)` throughout the codebase.
- System font discovery is now recursive (walkdir, follow symlinks) and picks up `.ttc`/`.otc` collections in addition to `.ttf`/`.otf`, matching how fonts are actually organised on macOS/Linux.

## [0.1.0] - 2026-05-14

Initial Release
