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

pub fn status(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;

    let loaded_map: std::collections::HashMap<String, (bool, Option<u32>)> =
        launchctl::list_loaded(user)?
            .into_iter()
            .map(|e| (e.label, (e.running, e.pid)))
            .collect();

    let (loaded, running, pid) = if let Some(&(running, pid)) = loaded_map.get(&agent.label) {
        (true, running, pid)
    } else {
        (false, false, None)
    };

    println!("{}: {}", "label".bold(), agent.label);
    println!("{}: {}", "path".bold(), agent.path.display());
    println!(
        "{}: {}",
        "loaded".bold(),
        if loaded {
            "yes".green().to_string()
        } else {
            "no".yellow().to_string()
        }
    );
    println!(
        "{}: {}",
        "running".bold(),
        if running {
            "yes".green().to_string()
        } else {
            "no".dimmed().to_string()
        }
    );
    println!(
        "{}: {}",
        "pid".bold(),
        pid.map(|p| p.to_string())
            .unwrap_or_else(|| "-".dimmed().to_string())
    );

    if !loaded {
        std::process::exit(1);
    }

    Ok(())
}

pub fn enable(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::load(&agent.path)?;
    state::write_hash(&agent.label, &agent.path)?;
    println!("{} enabled {}", "✓".green(), agent.label);
    Ok(())
}

pub fn disable(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::unload(&agent.path)?;
    state::delete_hash(&agent.label)?;
    println!("{} disabled {}", "✓".green(), agent.label);
    Ok(())
}

pub fn start(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::start(&agent.label)?;
    println!("{} started {}", "✓".green(), agent.label);
    Ok(())
}

pub fn stop(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::stop(&agent.label)?;
    println!("{} stopped {}", "✓".green(), agent.label);
    Ok(())
}

pub fn restart(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::stop(&agent.label)?;
    launchctl::start(&agent.label)?;
    println!("{} restarted {}", "✓".green(), agent.label);
    Ok(())
}

pub fn reload(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;

    // Hash check — skip if plist hasn't changed
    let current_hash = state::compute_hash(&agent.path)?;
    let stored_hash = state::read_hash(&agent.label)?;

    if stored_hash.as_deref() == Some(current_hash.as_str()) {
        println!("No changes detected in {}, skipping.", agent.label.dimmed());
        return Ok(());
    }

    // Check running state before unloading
    let loaded_map: std::collections::HashMap<String, (bool, Option<u32>)> =
        launchctl::list_loaded(user)?
            .into_iter()
            .map(|e| (e.label, (e.running, e.pid)))
            .collect();
    let was_running = loaded_map
        .get(&agent.label)
        .map(|(r, _)| *r)
        .unwrap_or(false);

    // Unload and reload without changing disabled/enabled state
    launchctl::unload_quiet(&agent.path)?;
    launchctl::load_quiet(&agent.path)?;

    if was_running {
        launchctl::start(&agent.label)?;
        println!("{} reloaded and restarted {}", "✓".green(), agent.label);
    } else {
        println!(
            "{} reloaded {} (not restarted, was not running)",
            "✓".green(),
            agent.label
        );
    }

    state::write_hash(&agent.label, &agent.path)?;
    Ok(())
}

pub fn validate(agent_id: &str, user: bool) -> Result<()> {
    todo!("validate - implemented in Task 13")
}
