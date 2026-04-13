use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;

use crate::{agent, launchctl, scope, state, validate as validate_mod};

pub fn list(user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;

    if !dir.exists() {
        eprintln!("directory does not exist: {}", dir.display());
        return Ok(());
    }

    // Build label -> (running, pid) map from launchctl
    let loaded_map: HashMap<String, (bool, Option<u32>)> = launchctl::list_loaded(user)?
        .into_iter()
        .map(|e| (e.label, (e.running, e.pid)))
        .collect();

    // Collect all plists from directory
    let mut rows: Vec<(String, bool, bool, Option<u32>)> = Vec::new();

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("plist") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let label = agent::read_label(&path).unwrap_or(stem);
        let (running, pid) = loaded_map
            .get(&label)
            .copied()
            .unwrap_or((false, None));
        let loaded = loaded_map.contains_key(&label);
        rows.push((label, loaded, running, pid));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));

    println!(
        "{:<45} {:<8} {:<8} {}",
        "LABEL".bold(),
        "LOADED".bold(),
        "RUNNING".bold(),
        "PID".bold()
    );

    for (label, loaded, running, pid) in rows {
        let loaded_str = if loaded {
            "yes".green().to_string()
        } else {
            "no".yellow().to_string()
        };
        let running_str = if running {
            "yes".green().to_string()
        } else {
            "no".dimmed().to_string()
        };
        let pid_str = pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".dimmed().to_string());
        println!("{:<45} {:<8} {:<8} {}", label, loaded_str, running_str, pid_str);
    }

    Ok(())
}

pub fn status(_agent: &str, _user: bool) -> Result<()> {
    todo!("status - implemented in Task 9")
}

pub fn enable(_agent: &str, _user: bool) -> Result<()> {
    todo!("enable - implemented in Task 10")
}

pub fn disable(_agent: &str, _user: bool) -> Result<()> {
    todo!("disable - implemented in Task 10")
}

pub fn start(_agent: &str, _user: bool) -> Result<()> {
    todo!("start - implemented in Task 11")
}

pub fn stop(_agent: &str, _user: bool) -> Result<()> {
    todo!("stop - implemented in Task 11")
}

pub fn restart(_agent: &str, _user: bool) -> Result<()> {
    todo!("restart - implemented in Task 11")
}

pub fn reload(_agent: &str, _user: bool) -> Result<()> {
    todo!("reload - implemented in Task 12")
}

pub fn validate(agent_id: &str, user: bool) -> Result<()> {
    todo!("validate - implemented in Task 13")
}
