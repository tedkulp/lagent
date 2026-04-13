use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};

/// An entry from `launchctl list` output.
pub struct LaunchctlEntry {
    pub label: String,
    pub running: bool,
    pub pid: Option<u32>,
}

/// Load an agent permanently (sets it to start at login).
pub fn load(path: &Path) -> Result<()> {
    run(&["load", "-w", path_str(path)?])
}

/// Unload an agent permanently (prevents it from starting at login).
pub fn unload(path: &Path) -> Result<()> {
    run(&["unload", "-w", path_str(path)?])
}

/// Load an agent without changing its disabled/enabled state.
pub fn load_quiet(path: &Path) -> Result<()> {
    run(&["load", path_str(path)?])
}

/// Unload an agent without changing its disabled/enabled state.
pub fn unload_quiet(path: &Path) -> Result<()> {
    run(&["unload", path_str(path)?])
}

/// Start an agent by label.
pub fn start(label: &str) -> Result<()> {
    run(&["start", label])
}

/// Stop an agent by label.
pub fn stop(label: &str) -> Result<()> {
    run(&["stop", label])
}

/// Return all loaded agents from `launchctl list`.
/// When running as root via sudo, queries the original user's session via $SUDO_USER.
pub fn list_loaded(user_scope: bool) -> Result<Vec<LaunchctlEntry>> {
    let output = if !user_scope {
        if let Some(sudo_user) = std::env::var("SUDO_USER").ok().filter(|u| !u.is_empty()) {
            Command::new("sudo")
                .args(["-u", &sudo_user, "launchctl", "list"])
                .output()
                .context("failed to run launchctl list as SUDO_USER")?
        } else {
            Command::new("launchctl")
                .arg("list")
                .output()
                .context("failed to run launchctl list")?
        }
    } else {
        Command::new("launchctl")
            .arg("list")
            .output()
            .context("failed to run launchctl list")?
    };

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines().skip(1) {
        // Format: PID \t Status \t Label
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() != 3 {
            continue;
        }
        let pid_str = parts[0].trim();
        let label = parts[2].trim().to_string();
        let pid = pid_str.parse::<u32>().ok();
        entries.push(LaunchctlEntry {
            label,
            running: pid.is_some(),
            pid,
        });
    }

    Ok(entries)
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("path contains non-UTF-8 characters: {}", path.display()))
}

fn run(args: &[&str]) -> Result<()> {
    let output = Command::new("launchctl")
        .args(args)
        .output()
        .with_context(|| format!("failed to run launchctl {}", args.join(" ")))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);
    anyhow::bail!(
        "launchctl {} failed (exit {}): {}",
        args.join(" "),
        code,
        stderr.trim()
    )
}
