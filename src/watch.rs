use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::config::ResolvedProfile;
use crate::convert::{self, is_supported_source};
use crate::error::{Result, TyposError};
use crate::output;

const DEBOUNCE: Duration = Duration::from_millis(250);

/// Watch `path` and re-convert any matching source files whose contents change.
pub(crate) fn run(
    path: &Path,
    profiles: &[ResolvedProfile],
    output_override: Option<&Path>,
) -> Result<()> {
    if !path.exists() {
        return Err(TyposError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("watch target does not exist: {}", path.display()),
        )));
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)
        .map_err(|e| TyposError::Io(std::io::Error::other(format!("watcher init: {e}"))))?;
    let mode = if path.is_dir() {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    let watch_root = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    watcher
        .watch(watch_root, mode)
        .map_err(|e| TyposError::Io(std::io::Error::other(format!("watch: {e}"))))?;

    output::header(&format!("Watching {}", output::short_path(path)));
    output::info("press Ctrl+C to stop");

    // Initial conversion so the user sees something immediately.
    convert_target(path, profiles, output_override);

    // Per-path debounce so a burst of rename/save events compiles once.
    let mut last_fired: std::collections::HashMap<PathBuf, Instant> = Default::default();

    while let Ok(event) = rx.recv() {
        let event = match event {
            Ok(e) => e,
            Err(e) => {
                output::warn(&format!("watch error: {e}"));
                continue;
            }
        };
        if !is_change_event(&event.kind) {
            continue;
        }
        for changed in event.paths {
            if !is_supported_source(&changed) {
                continue;
            }
            // If the user passed a single file, ignore changes to siblings.
            if path.is_file() && changed.file_name() != path.file_name() {
                continue;
            }
            let now = Instant::now();
            if let Some(prev) = last_fired.get(&changed)
                && now.duration_since(*prev) < DEBOUNCE
            {
                continue;
            }
            last_fired.insert(changed.clone(), now);
            convert_target(&changed, profiles, output_override);
        }
    }
    Ok(())
}

fn is_change_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn convert_target(path: &Path, profiles: &[ResolvedProfile], output_override: Option<&Path>) {
    if path.is_dir() {
        let results = convert::batch(path, profiles, output_override);
        for (src, file_results) in &results {
            for (profile_name, r) in file_results {
                convert::print_result(src, profile_name, r);
            }
        }
        return;
    }
    if !path.is_file() || !is_supported_source(path) {
        return;
    }
    let results = convert::convert_file_multi(path, profiles, output_override);
    for (profile_name, r) in &results {
        convert::print_result(path, profile_name, r);
    }
}
