use console::{Term, style};
use std::path::Path;

pub(crate) fn ok(name: &str, msg: &str) {
    let _ = Term::stdout().write_line(&format!(
        "  {} {}  {}",
        style("✓").green().bold(),
        style(name).bold(),
        style(msg).dim(),
    ));
}

pub(crate) fn fail(name: &str, context: &str, msg: &str) {
    let _ = Term::stderr().write_line(&format!(
        "  {} {}  [{}]: {}",
        style("!!").red().bold(),
        style(name).bold(),
        style(context).yellow(),
        style(msg).red(),
    ));
}

pub(crate) fn warn(msg: &str) {
    let _ = Term::stderr().write_line(&format!(
        "  {} {}",
        style("!").yellow().bold(),
        msg,
    ));
}

pub(crate) fn header(msg: &str) {
    let _ = Term::stdout().write_line(&format!("{}", style(msg).bold().underlined()));
}

pub(crate) fn info(msg: &str) {
    let _ = Term::stdout().write_line(&format!("  {}", style(msg).dim()));
}

/// Trim an absolute path to at most `parent/filename` for compact output.
pub(crate) fn short_path(path: &Path) -> String {
    let file = path.file_name().map(|n| n.to_string_lossy());
    let parent = path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy());
    match (parent, file) {
        (Some(p), Some(f)) => format!("{}/{}", p, f),
        (None, Some(f)) => f.into_owned(),
        _ => path.to_string_lossy().into_owned(),
    }
}
