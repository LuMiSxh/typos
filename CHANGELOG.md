# Changelog

All notable changes to this project will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
