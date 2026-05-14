use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::path::PathBuf;
use crate::config::ResolvedProfile;
use crate::error::{Result, TyposError};

/// Show a profile picker when multiple profiles exist and none was specified via flag.
/// Returns the selected profiles (one or more).
pub fn pick_profiles<'a>(profiles: &'a [ResolvedProfile]) -> Result<Vec<&'a ResolvedProfile>> {
    if profiles.is_empty() {
        return Err(TyposError::NoProfiles);
    }
    if profiles.len() == 1 {
        return Ok(vec![&profiles[0]]);
    }

    let names: Vec<String> = profiles.iter()
        .map(|p| format!("{} ({})", p.display_name, p.name))
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select profiles (space to toggle, enter to confirm)")
        .items(&names)
        .interact()
        .map_err(|e| TyposError::Io(std::io::Error::other(e.to_string())))?;

    if selections.is_empty() {
        return Err(TyposError::NoProfiles);
    }

    Ok(selections.iter().map(|&i| &profiles[i]).collect())
}

#[derive(Debug)]
pub enum GuidedAction {
    ConvertFile { path: PathBuf, profiles: Vec<String> },
    Batch { dir: PathBuf, profiles: Vec<String> },
    List,
}

/// Full guided no-argument flow.
pub fn guided_flow(profiles: &[ResolvedProfile]) -> Result<GuidedAction> {
    if profiles.is_empty() {
        return Err(TyposError::NoProfiles);
    }

    const ACTION_CONVERT: usize = 0;
    const ACTION_BATCH: usize = 1;
    const ACTION_LIST: usize = 2;

    let actions = ["Convert a single file", "Convert a directory (batch)", "List profiles"];
    let action = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to do?")
        .items(&actions)
        .default(0)
        .interact()
        .map_err(|e| TyposError::Io(std::io::Error::other(e.to_string())))?;

    if action == ACTION_LIST {
        return Ok(GuidedAction::List);
    }

    let path_str: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(if action == ACTION_CONVERT { "Markdown file path" } else { "Directory path" })
        .interact_text()
        .map_err(|e| TyposError::Io(std::io::Error::other(e.to_string())))?;

    let path = PathBuf::from(path_str.trim());

    let profile_names: Vec<String> = profiles.iter()
        .map(|p| format!("{} ({})", p.display_name, p.name))
        .collect();

    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select profiles")
        .items(&profile_names)
        .interact()
        .map_err(|e| TyposError::Io(std::io::Error::other(e.to_string())))?;

    let profile_indices: Vec<String> = if selected.is_empty() {
        vec![profiles[0].name.clone()]
    } else {
        selected.iter().map(|&i| profiles[i].name.clone()).collect()
    };

    if action == ACTION_CONVERT {
        Ok(GuidedAction::ConvertFile { path, profiles: profile_indices })
    } else if action == ACTION_BATCH {
        Ok(GuidedAction::Batch { dir: path, profiles: profile_indices })
    } else {
        // Should not reach here given the 3-item actions array, but be explicit
        Ok(GuidedAction::List)
    }
}
