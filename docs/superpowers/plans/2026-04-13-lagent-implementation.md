# lagent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a single Rust binary `lagent` that manages macOS LaunchAgents with systemctl-style commands.

**Architecture:** Hybrid approach — use `launchctl` subprocesses for all mutations (load/unload/start/stop), use the `plist` crate to scan directories for listing and status, and store SHA256 hashes of plist files to avoid unnecessary reloads. A `commands.rs` module holds all command implementations; other modules handle discrete concerns.

**Tech Stack:** Rust stable, clap v4 (derive), plist, sha2, anyhow, owo-colors, clap_complete; macOS system tools launchctl and plutil.

---

## File Map

| File | Responsibility |
|---|---|
| `Cargo.toml` | Dependencies and release profile |
| `Makefile` | build/install targets |
| `src/main.rs` | clap entrypoint + command dispatch |
| `src/cli.rs` | clap derive structs for all commands |
| `src/scope.rs` | Resolve target directory, check root |
| `src/agent.rs` | Resolve agent by label or filename |
| `src/launchctl.rs` | Subprocess wrappers for launchctl |
| `src/state.rs` | SHA256 hash read/write/delete |
| `src/validate.rs` | plutil syntax + plist schema checks |
| `src/commands.rs` | All command implementations (list, status, enable, disable, start, stop, restart, reload, validate, completions) |

> Note: `commands.rs` is added beyond the spec's structure to keep `main.rs` clean. All logic still maps 1:1 to the spec.

---

## Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `Makefile`
- Create: `src/main.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "lagent"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "lagent"
path = "src/main.rs"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
clap_complete = "4"
owo-colors = "4"
plist = "1"
sha2 = "1"

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = 3
strip = true
lto = true
codegen-units = 1
```

- [ ] **Step 2: Create Makefile**

```makefile
.PHONY: build install clean

build:
	cargo build --release

install: build
	install -m 755 target/release/lagent /usr/local/bin/lagent

clean:
	cargo clean
```

- [ ] **Step 3: Create src/main.rs (minimal stub)**

```rust
fn main() {
    println!("lagent");
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build
```

Expected: compiles without errors, binary at `target/debug/lagent`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Makefile src/main.rs
git commit -m "feat: project scaffold"
```

---

## Task 2: CLI Structure

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create src/cli.rs**

```rust
use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "lagent",
    about = "Manage macOS LaunchAgents",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Target ~/Library/LaunchAgents instead of /Library/LaunchAgents
    #[arg(long, global = true)]
    pub user: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all LaunchAgents in the target directory
    List,
    /// Show status of a specific agent
    Status {
        /// Agent label or plist filename (with or without .plist)
        agent: String,
    },
    /// Load and permanently enable an agent to start at login
    Enable {
        /// Agent label or plist filename
        agent: String,
    },
    /// Permanently disable and unload an agent
    Disable {
        /// Agent label or plist filename
        agent: String,
    },
    /// Start a loaded agent immediately
    Start {
        /// Agent label or plist filename
        agent: String,
    },
    /// Stop a running agent immediately
    Stop {
        /// Agent label or plist filename
        agent: String,
    },
    /// Stop then start an agent (does not reload plist)
    Restart {
        /// Agent label or plist filename
        agent: String,
    },
    /// Reload plist definition from disk; restart if was running and plist changed
    Reload {
        /// Agent label or plist filename
        agent: String,
    },
    /// Validate a plist for syntax and launchd schema correctness
    Validate {
        /// Agent label or plist filename
        agent: String,
    },
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}
```

- [ ] **Step 2: Update src/main.rs to use clap**

```rust
mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    let _ = cli; // commands implemented in later tasks
    println!("TODO: dispatch {:?}", std::mem::discriminant(&cli.command));
}
```

- [ ] **Step 3: Verify --help works**

```bash
cargo run -- --help
```

Expected output includes: `list`, `status`, `enable`, `disable`, `start`, `stop`, `restart`, `reload`, `validate`, `completions` subcommands listed.

```bash
cargo run -- list --help
```

Expected: shows `--user` flag.

- [ ] **Step 4: Commit**

```bash
git add src/cli.rs src/main.rs Cargo.lock
git commit -m "feat: CLI structure with all commands"
```

---

## Task 3: Scope Module

**Files:**
- Create: `src/scope.rs`
- Modify: `src/main.rs` (add `mod scope;`)

- [ ] **Step 1: Write the failing tests in src/scope.rs**

```rust
use std::path::PathBuf;
use anyhow::Result;

pub fn target_dir(user: bool) -> Result<PathBuf> {
    todo!()
}

pub fn check_root() -> Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_scope_returns_home_library_path() {
        // HOME must be set in test environment
        std::env::set_var("HOME", "/Users/testuser");
        let path = target_dir(true).unwrap();
        assert_eq!(path, PathBuf::from("/Users/testuser/Library/LaunchAgents"));
    }

    #[test]
    fn test_system_scope_returns_library_path() {
        // This test only checks the path, not the root check.
        // We call the internal path builder directly.
        let path = system_agents_dir();
        assert_eq!(path, PathBuf::from("/Library/LaunchAgents"));
    }

    #[test]
    fn test_is_root_returns_bool() {
        // Just verify it returns a bool without panicking
        let _ = is_root();
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test scope
```

Expected: fails with "not yet implemented" panics.

- [ ] **Step 3: Implement src/scope.rs**

```rust
use std::path::PathBuf;
use anyhow::Result;

pub fn target_dir(user: bool) -> Result<PathBuf> {
    if user {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        Ok(PathBuf::from(home).join("Library/LaunchAgents"))
    } else {
        check_root()?;
        Ok(system_agents_dir())
    }
}

pub fn check_root() -> Result<()> {
    if !is_root() {
        anyhow::bail!("this operation requires root. Re-run with sudo.");
    }
    Ok(())
}

pub fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() == "0")
        .unwrap_or(false)
}

pub fn system_agents_dir() -> PathBuf {
    PathBuf::from("/Library/LaunchAgents")
}

/// When running as root via sudo, $SUDO_USER holds the original username.
/// Returns it so we can query the right launchctl session.
pub fn sudo_user() -> Option<String> {
    std::env::var("SUDO_USER").ok().filter(|u| !u.is_empty())
}
```

- [ ] **Step 4: Add `mod scope;` to src/main.rs**

Add after `mod cli;`:
```rust
mod scope;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test scope
```

Expected: all 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/scope.rs src/main.rs
git commit -m "feat: scope module with root check and path resolution"
```

---

## Task 4: Agent Resolution Module

**Files:**
- Create: `src/agent.rs`
- Modify: `src/main.rs` (add `mod agent;`)

- [ ] **Step 1: Write failing tests in src/agent.rs**

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct Agent {
    pub path: PathBuf,
    pub label: String,
}

pub fn resolve(id: &str, dir: &Path) -> Result<Agent> {
    todo!()
}

fn read_label(path: &Path) -> Result<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_plist(dir: &Path, stem: &str, label: &str) -> PathBuf {
        let path = dir.join(format!("{}.plist", stem));
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
</dict>
</plist>"#
        );
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_resolve_by_stem() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp", dir.path()).unwrap();
        assert_eq!(agent.label, "com.example.myapp");
    }

    #[test]
    fn test_resolve_strips_plist_suffix() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp.plist", dir.path()).unwrap();
        assert_eq!(agent.label, "com.example.myapp");
    }

    #[test]
    fn test_resolve_by_label_when_stem_differs() {
        let dir = TempDir::new().unwrap();
        write_plist(dir.path(), "myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp", dir.path()).unwrap();
        assert_eq!(agent.label, "com.example.myapp");
    }

    #[test]
    fn test_resolve_no_match_returns_error() {
        let dir = TempDir::new().unwrap();
        let result = resolve("nonexistent", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no agent found"));
    }

    #[test]
    fn test_resolve_ambiguous_returns_error() {
        let dir = TempDir::new().unwrap();
        // Two plists, both with the same label
        write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        write_plist(dir.path(), "com.example.myapp-alt", "com.example.myapp");
        let result = resolve("com.example.myapp", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ambiguous"));
    }

    #[test]
    fn test_resolve_path_is_correct() {
        let dir = TempDir::new().unwrap();
        let expected_path = write_plist(dir.path(), "com.example.myapp", "com.example.myapp");
        let agent = resolve("com.example.myapp", dir.path()).unwrap();
        assert_eq!(agent.path, expected_path);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test agent
```

Expected: fails with "not yet implemented".

- [ ] **Step 3: Implement src/agent.rs**

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct Agent {
    pub path: PathBuf,
    pub label: String,
}

/// Resolve an agent by label or filename stem in the given directory.
pub fn resolve(id: &str, dir: &Path) -> Result<Agent> {
    let needle = id.trim_end_matches(".plist");
    let mut matches: Vec<Agent> = Vec::new();

    for entry in std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("cannot read directory {}: {}", dir.display(), e))?
    {
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

        if stem == needle {
            let label = read_label(&path).unwrap_or_else(|_| stem.clone());
            matches.push(Agent { path, label });
            continue;
        }

        if let Ok(label) = read_label(&path) {
            if label == needle {
                matches.push(Agent { path, label });
            }
        }
    }

    match matches.len() {
        0 => anyhow::bail!("no agent found matching '{}'", id),
        1 => Ok(matches.remove(0)),
        _ => {
            let names: Vec<_> = matches.iter().map(|a| a.label.as_str()).collect();
            anyhow::bail!(
                "ambiguous agent name '{}', matches: {}",
                id,
                names.join(", ")
            )
        }
    }
}

/// Read the Label key from a plist file.
pub fn read_label(path: &Path) -> Result<String> {
    let value: plist::Value = plist::from_file(path)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {}", path.display(), e))?;
    let dict = value
        .as_dictionary()
        .ok_or_else(|| anyhow::anyhow!("plist root is not a dictionary"))?;
    dict.get("Label")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Label key not found or not a string"))
}
```

- [ ] **Step 4: Add `mod agent;` to src/main.rs**

```rust
mod agent;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test agent
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/agent.rs src/main.rs
git commit -m "feat: agent resolution by label or filename"
```

---

## Task 5: State Module

**Files:**
- Create: `src/state.rs`
- Modify: `src/main.rs` (add `mod state;`)

- [ ] **Step 1: Write failing tests in src/state.rs**

```rust
use std::path::Path;
use anyhow::Result;
use sha2::{Digest, Sha256};

pub fn compute_hash(plist_path: &Path) -> Result<String> {
    todo!()
}

pub fn read_hash(label: &str) -> Result<Option<String>> {
    todo!()
}

pub fn write_hash(label: &str, plist_path: &Path) -> Result<()> {
    todo!()
}

pub fn delete_hash(label: &str) -> Result<()> {
    todo!()
}

fn state_dir() -> Result<std::path::PathBuf> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup(tmp: &TempDir) -> std::path::PathBuf {
        let state = tmp.path().join("state");
        std::fs::create_dir_all(&state).unwrap();
        std::env::set_var("LAGENT_STATE_DIR", &state);
        let plist = tmp.path().join("test.plist");
        std::fs::write(&plist, b"<plist>test</plist>").unwrap();
        plist
    }

    #[test]
    fn test_compute_hash_is_deterministic() {
        let tmp = TempDir::new().unwrap();
        let plist = tmp.path().join("a.plist");
        std::fs::write(&plist, b"hello").unwrap();
        let h1 = compute_hash(&plist).unwrap();
        let h2 = compute_hash(&plist).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_hash_differs_for_different_content() {
        let tmp = TempDir::new().unwrap();
        let p1 = tmp.path().join("a.plist");
        let p2 = tmp.path().join("b.plist");
        std::fs::write(&p1, b"hello").unwrap();
        std::fs::write(&p2, b"world").unwrap();
        assert_ne!(compute_hash(&p1).unwrap(), compute_hash(&p2).unwrap());
    }

    #[test]
    fn test_read_hash_returns_none_when_missing() {
        let tmp = TempDir::new().unwrap();
        setup(&tmp);
        let result = read_hash("com.example.nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_write_then_read_hash() {
        let tmp = TempDir::new().unwrap();
        let plist = setup(&tmp);
        write_hash("com.example.test", &plist).unwrap();
        let stored = read_hash("com.example.test").unwrap().unwrap();
        let expected = compute_hash(&plist).unwrap();
        assert_eq!(stored, expected);
    }

    #[test]
    fn test_delete_hash_removes_file() {
        let tmp = TempDir::new().unwrap();
        let plist = setup(&tmp);
        write_hash("com.example.test", &plist).unwrap();
        assert!(read_hash("com.example.test").unwrap().is_some());
        delete_hash("com.example.test").unwrap();
        assert!(read_hash("com.example.test").unwrap().is_none());
    }

    #[test]
    fn test_delete_hash_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        setup(&tmp);
        // Deleting a non-existent hash should not error
        delete_hash("com.example.nonexistent").unwrap();
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test state
```

Expected: fails with "not yet implemented".

- [ ] **Step 3: Implement src/state.rs**

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;
use sha2::{Digest, Sha256};

fn state_dir() -> Result<PathBuf> {
    let dir = if let Ok(d) = std::env::var("LAGENT_STATE_DIR") {
        PathBuf::from(d)
    } else {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        PathBuf::from(home).join(".local/share/lagent")
    };
    std::fs::create_dir_all(&dir)
        .map_err(|e| anyhow::anyhow!("cannot create state directory {}: {}", dir.display(), e))?;
    Ok(dir)
}

fn hash_path(label: &str) -> Result<PathBuf> {
    Ok(state_dir()?.join(format!("{}.hash", label)))
}

/// Compute SHA256 of a plist file's raw bytes.
pub fn compute_hash(plist_path: &Path) -> Result<String> {
    let bytes = std::fs::read(plist_path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {}", plist_path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Read stored hash for a label. Returns None if no hash exists.
pub fn read_hash(label: &str) -> Result<Option<String>> {
    let path = hash_path(label)?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("cannot read hash file: {}", e))?;
        Ok(Some(content.trim().to_string()))
    } else {
        Ok(None)
    }
}

/// Write the hash of a plist file for a label.
pub fn write_hash(label: &str, plist_path: &Path) -> Result<()> {
    let hash = compute_hash(plist_path)?;
    std::fs::write(hash_path(label)?, hash)
        .map_err(|e| anyhow::anyhow!("cannot write hash file: {}", e))
}

/// Delete the stored hash for a label. No-op if it doesn't exist.
pub fn delete_hash(label: &str) -> Result<()> {
    let path = hash_path(label)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| anyhow::anyhow!("cannot delete hash file: {}", e))?;
    }
    Ok(())
}
```

- [ ] **Step 4: Add `mod state;` to src/main.rs**

```rust
mod state;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test state
```

Expected: all 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/state.rs src/main.rs
git commit -m "feat: state module for plist hash storage"
```

---

## Task 6: launchctl Subprocess Wrappers

**Files:**
- Create: `src/launchctl.rs`
- Modify: `src/main.rs` (add `mod launchctl;`)

No unit tests for this module — it wraps OS commands. Manual verification in Task 8+.

- [ ] **Step 1: Create src/launchctl.rs**

```rust
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
/// When running as root via sudo, queries the original user's session.
pub fn list_loaded(user_scope: bool) -> Result<Vec<LaunchctlEntry>> {
    let output = if !user_scope {
        // System scope: we're root. $SUDO_USER holds the logged-in user.
        // We need their session to see which LaunchAgents are running.
        if let Ok(sudo_user) = std::env::var("SUDO_USER").ok().filter(|u| !u.is_empty()) {
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
```

- [ ] **Step 2: Add `mod launchctl;` to src/main.rs**

```rust
mod launchctl;
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/launchctl.rs src/main.rs
git commit -m "feat: launchctl subprocess wrappers"
```

---

## Task 7: Validate Module

**Files:**
- Create: `src/validate.rs`
- Modify: `src/main.rs` (add `mod validate;`)

- [ ] **Step 1: Write failing tests in src/validate.rs**

```rust
use std::path::Path;
use anyhow::Result;

pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn validate(path: &Path) -> Result<ValidationResult> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_valid_plist(dir: &Path, label: &str) -> std::path::PathBuf {
        let path = dir.join("test.plist");
        fs::write(&path, format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
</dict>
</plist>"#
        )).unwrap();
        path
    }

    fn write_plist_without_program(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("noprog.plist");
        fs::write(&path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.test</string>
</dict>
</plist>"#
        ).unwrap();
        path
    }

    fn write_plist_without_label(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("nolabel.plist");
        fs::write(&path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
</dict>
</plist>"#
        ).unwrap();
        path
    }

    fn write_plist_with_unknown_key(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("unknown.plist");
        fs::write(&path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.test</string>
    <key>ProgramArguments</key>
    <array><string>/usr/bin/true</string></array>
    <key>SuperUnknownKey</key>
    <string>value</string>
</dict>
</plist>"#
        ).unwrap();
        path
    }

    #[test]
    fn test_valid_plist_has_no_errors() {
        let tmp = TempDir::new().unwrap();
        let path = write_valid_plist(tmp.path(), "com.example.test");
        let result = validate(&path).unwrap();
        assert!(result.errors.is_empty(), "expected no errors, got: {:?}", result.errors);
    }

    #[test]
    fn test_missing_program_is_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_plist_without_program(tmp.path());
        let result = validate(&path).unwrap();
        assert!(result.errors.iter().any(|e| e.contains("Program")));
    }

    #[test]
    fn test_missing_label_is_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_plist_without_label(tmp.path());
        let result = validate(&path).unwrap();
        assert!(result.errors.iter().any(|e| e.contains("Label")));
    }

    #[test]
    fn test_unknown_key_is_warning_not_error() {
        let tmp = TempDir::new().unwrap();
        let path = write_plist_with_unknown_key(tmp.path());
        let result = validate(&path).unwrap();
        assert!(result.errors.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("SuperUnknownKey")));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test validate
```

Expected: fails with "not yet implemented".

- [ ] **Step 3: Implement src/validate.rs**

```rust
use std::path::Path;
use std::process::Command;
use anyhow::Result;

pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Standard launchd plist keys. Unknown keys produce warnings.
const KNOWN_KEYS: &[&str] = &[
    "Label",
    "Program",
    "ProgramArguments",
    "WorkingDirectory",
    "StandardOutPath",
    "StandardErrorPath",
    "EnvironmentVariables",
    "RunAtLoad",
    "StartInterval",
    "StartCalendarInterval",
    "KeepAlive",
    "OnDemand",
    "UserName",
    "GroupName",
    "Disabled",
    "LimitLoadToSessionType",
    "AbandonProcessGroup",
    "WatchPaths",
    "QueueDirectories",
    "SoftResourceLimits",
    "HardResourceLimits",
    "Nice",
    "ProcessType",
    "ThrottleInterval",
    "ExitTimeout",
    "TimeOut",
    "inetdCompatibility",
    "Sockets",
    "SessionCreate",
    "Debug",
    "WaitForDebugger",
    "EnableGlobbing",
    "EnableTransactions",
    "LaunchOnlyOnce",
    "MachServices",
    "BundleProgram",
];

pub fn validate(path: &Path) -> Result<ValidationResult> {
    let mut result = ValidationResult {
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Stage 1: syntax via plutil
    let output = Command::new("plutil")
        .args(["-lint", path.to_str().unwrap_or("")])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run plutil: {}", e))?;

    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        result.errors.push(format!("syntax error: {}", msg.trim()));
        return Ok(result);
    }

    // Stage 2: schema via plist crate
    let value: plist::Value = plist::from_file(path)
        .map_err(|e| anyhow::anyhow!("failed to parse plist: {}", e))?;

    let dict = match value.as_dictionary() {
        Some(d) => d,
        None => {
            result.errors.push("plist root must be a dictionary".to_string());
            return Ok(result);
        }
    };

    // Required: Label (non-empty string)
    match dict.get("Label").and_then(|v| v.as_string()) {
        None => result.errors.push("missing required key: Label".to_string()),
        Some(l) if l.is_empty() => result
            .errors
            .push("Label must not be empty".to_string()),
        _ => {}
    }

    // Required: Program or ProgramArguments (at least one)
    if dict.get("Program").is_none() && dict.get("ProgramArguments").is_none() {
        result
            .errors
            .push("missing required key: Program or ProgramArguments".to_string());
    }

    // Warn on unknown top-level keys
    for key in dict.keys() {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            result.warnings.push(format!("unknown key: {}", key));
        }
    }

    Ok(result)
}
```

- [ ] **Step 4: Add `mod validate;` to src/main.rs**

```rust
mod validate;
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test validate
```

Expected: all 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/validate.rs src/main.rs
git commit -m "feat: validate module with plutil + plist schema checks"
```

---

## Task 8: Commands Module + list Command

**Files:**
- Create: `src/commands.rs`
- Modify: `src/main.rs` (add `mod commands;`, wire up dispatch)

- [ ] **Step 1: Create src/commands.rs with list command**

```rust
use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;

use crate::{agent, launchctl, scope};

pub fn list(user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;

    if !dir.exists() {
        eprintln!("directory does not exist: {}", dir.display());
        return Ok(());
    }

    // Build map of label -> (running, pid) from launchctl
    let loaded_map: HashMap<String, (bool, Option<u32>)> = launchctl::list_loaded(user)?
        .into_iter()
        .map(|e| (e.label, (e.running, e.pid)))
        .collect();

    // Collect all plists from the directory
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
        let pid_str = pid.map(|p| p.to_string()).unwrap_or_else(|| "-".dimmed().to_string());
        println!("{:<45} {:<8} {:<8} {}", label, loaded_str, running_str, pid_str);
    }

    Ok(())
}
```

- [ ] **Step 2: Update src/main.rs with full dispatch**

```rust
mod agent;
mod cli;
mod commands;
mod launchctl;
mod scope;
mod state;
mod validate;

use anyhow::Result;
use clap::Parser;
use clap_complete::generate;
use cli::{Cli, Commands};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::list(cli.user),
        Commands::Status { agent } => commands::status(&agent, cli.user),
        Commands::Enable { agent } => commands::enable(&agent, cli.user),
        Commands::Disable { agent } => commands::disable(&agent, cli.user),
        Commands::Start { agent } => commands::start(&agent, cli.user),
        Commands::Stop { agent } => commands::stop(&agent, cli.user),
        Commands::Restart { agent } => commands::restart(&agent, cli.user),
        Commands::Reload { agent } => commands::reload(&agent, cli.user),
        Commands::Validate { agent } => commands::validate(&agent, cli.user),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "lagent", &mut std::io::stdout());
            Ok(())
        }
    }
}
```

Add to imports at top of main.rs:
```rust
use owo_colors::OwoColorize;
use clap::CommandFactory;
```

- [ ] **Step 3: Add stub implementations for all other commands to commands.rs**

Add after the `list` function, so the file compiles:

```rust
pub fn status(_agent: &str, _user: bool) -> Result<()> {
    todo!("status")
}
pub fn enable(_agent: &str, _user: bool) -> Result<()> {
    todo!("enable")
}
pub fn disable(_agent: &str, _user: bool) -> Result<()> {
    todo!("disable")
}
pub fn start(_agent: &str, _user: bool) -> Result<()> {
    todo!("start")
}
pub fn stop(_agent: &str, _user: bool) -> Result<()> {
    todo!("stop")
}
pub fn restart(_agent: &str, _user: bool) -> Result<()> {
    todo!("restart")
}
pub fn reload(_agent: &str, _user: bool) -> Result<()> {
    todo!("reload")
}
pub fn validate(_agent: &str, _user: bool) -> Result<()> {
    todo!("validate")
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build
```

Expected: compiles, binary works.

- [ ] **Step 5: Manual smoke test of list (requires a LaunchAgent to be present)**

```bash
# User scope (no sudo needed)
cargo run -- list --user
```

Expected: table with LABEL/LOADED/RUNNING/PID columns, listing any agents in ~/Library/LaunchAgents.

- [ ] **Step 6: Commit**

```bash
git add src/commands.rs src/main.rs
git commit -m "feat: commands module + list command"
```

---

## Task 9: status Command

**Files:**
- Modify: `src/commands.rs`

- [ ] **Step 1: Replace the status stub in src/commands.rs**

```rust
pub fn status(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;

    let loaded_map: HashMap<String, (bool, Option<u32>)> = launchctl::list_loaded(user)?
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
        if loaded { "yes".green().to_string() } else { "no".yellow().to_string() }
    );
    println!(
        "{}: {}",
        "running".bold(),
        if running { "yes".green().to_string() } else { "no".dimmed().to_string() }
    );
    println!(
        "{}: {}",
        "pid".bold(),
        pid.map(|p| p.to_string()).unwrap_or_else(|| "-".dimmed().to_string())
    );

    if !loaded {
        std::process::exit(1);
    }

    Ok(())
}
```

- [ ] **Step 2: Manual smoke test**

```bash
cargo run -- status --user com.example.something
```

Expected: either shows status table, or error "no agent found matching 'com.example.something'".

- [ ] **Step 3: Commit**

```bash
git add src/commands.rs
git commit -m "feat: status command"
```

---

## Task 10: enable + disable Commands

**Files:**
- Modify: `src/commands.rs`

- [ ] **Step 1: Replace enable stub in src/commands.rs**

```rust
pub fn enable(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::load(&agent.path)?;
    state::write_hash(&agent.label, &agent.path)?;
    println!("{} enabled {}", "✓".green(), agent.label);
    Ok(())
}
```

- [ ] **Step 2: Replace disable stub in src/commands.rs**

```rust
pub fn disable(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::unload(&agent.path)?;
    state::delete_hash(&agent.label)?;
    println!("{} disabled {}", "✓".green(), agent.label);
    Ok(())
}
```

- [ ] **Step 3: Add state to imports at top of commands.rs**

```rust
use crate::{agent, launchctl, scope, state};
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 5: Commit**

```bash
git add src/commands.rs
git commit -m "feat: enable and disable commands"
```

---

## Task 11: start, stop, restart Commands

**Files:**
- Modify: `src/commands.rs`

- [ ] **Step 1: Replace start stub**

```rust
pub fn start(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::start(&agent.label)?;
    println!("{} started {}", "✓".green(), agent.label);
    Ok(())
}
```

- [ ] **Step 2: Replace stop stub**

```rust
pub fn stop(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::stop(&agent.label)?;
    println!("{} stopped {}", "✓".green(), agent.label);
    Ok(())
}
```

- [ ] **Step 3: Replace restart stub**

```rust
pub fn restart(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;
    launchctl::stop(&agent.label)?;
    launchctl::start(&agent.label)?;
    println!("{} restarted {}", "✓".green(), agent.label);
    Ok(())
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 5: Commit**

```bash
git add src/commands.rs
git commit -m "feat: start, stop, restart commands"
```

---

## Task 12: reload Command

**Files:**
- Modify: `src/commands.rs`

- [ ] **Step 1: Replace reload stub**

```rust
pub fn reload(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;

    // Compare current hash to stored hash
    let current_hash = state::compute_hash(&agent.path)?;
    let stored_hash = state::read_hash(&agent.label)?;

    if stored_hash.as_deref() == Some(current_hash.as_str()) {
        println!("No changes detected in {}, skipping.", agent.label.dimmed());
        return Ok(());
    }

    // Check if currently running before unloading
    let loaded_map: HashMap<String, (bool, Option<u32>)> = launchctl::list_loaded(user)?
        .into_iter()
        .map(|e| (e.label, (e.running, e.pid)))
        .collect();
    let was_running = loaded_map.get(&agent.label).map(|(r, _)| *r).unwrap_or(false);

    launchctl::unload_quiet(&agent.path)?;
    launchctl::load_quiet(&agent.path)?;

    if was_running {
        launchctl::start(&agent.label)?;
        println!("{} reloaded and restarted {}", "✓".green(), agent.label);
    } else {
        println!("{} reloaded {} (not restarted, was not running)", "✓".green(), agent.label);
    }

    state::write_hash(&agent.label, &agent.path)?;
    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/commands.rs
git commit -m "feat: reload command with hash-based change detection"
```

---

## Task 13: validate Command

**Files:**
- Modify: `src/commands.rs`

- [ ] **Step 1: Replace validate stub**

```rust
pub fn validate(agent_id: &str, user: bool) -> Result<()> {
    let dir = scope::target_dir(user)?;
    let agent = agent::resolve(agent_id, &dir)?;

    let result = crate::validate::validate(&agent.path)?;

    for warning in &result.warnings {
        eprintln!("{} {}", "warning:".yellow(), warning);
    }

    if result.errors.is_empty() {
        println!("{} {} is valid", "✓".green(), agent.path.display());
    } else {
        for error in &result.errors {
            eprintln!("{} {}", "error:".red(), error);
        }
        std::process::exit(1);
    }

    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/commands.rs
git commit -m "feat: validate command"
```

---

## Task 14: completions Command

**Files:**
- Modify: `src/main.rs` (already wired; just verify)

- [ ] **Step 1: Verify completions generate correctly**

```bash
cargo run -- completions zsh
```

Expected: zsh completion script printed to stdout (starts with `#compdef lagent`).

```bash
cargo run -- completions bash
```

Expected: bash completion script printed to stdout.

```bash
cargo run -- completions fish
```

Expected: fish completion script printed to stdout.

- [ ] **Step 2: Commit (no code change needed if already working)**

```bash
git commit --allow-empty -m "chore: verify completions for all shells"
```

---

## Task 15: Release Build + Install Verification

**Files:**
- No code changes

- [ ] **Step 1: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Build release binary**

```bash
cargo build --release
ls -lh target/release/lagent
```

Expected: single binary, under 5MB with `strip = true`.

- [ ] **Step 3: Verify the binary is self-contained**

```bash
otool -L target/release/lagent
```

Expected: only links to macOS system libraries (`libSystem`, `libresolv`, etc.) — no third-party `.dylib` dependencies.

- [ ] **Step 4: Smoke test the release binary**

```bash
./target/release/lagent --help
./target/release/lagent --version
./target/release/lagent list --user
./target/release/lagent completions zsh | head -5
```

Expected: all commands work correctly.

- [ ] **Step 5: Final commit and tag**

```bash
git add -A
git commit -m "chore: final build verification"
git tag v0.1.0
```

---

## Self-Review Checklist

| Spec Requirement | Covered in Task |
|---|---|
| enable command | Task 10 |
| disable command | Task 10 |
| start command | Task 11 |
| stop command | Task 11 |
| restart command | Task 11 |
| reload with hash check | Task 12 |
| validate (plutil + schema) | Task 7 + 13 |
| list command | Task 8 |
| status command | Task 9 |
| completions command | Task 14 |
| --user / system scope | Task 3 |
| root check with clear error | Task 3 |
| agent resolution by label or filename | Task 4 |
| SHA256 state storage | Task 5 |
| single compiled binary | Task 15 |
| Makefile install target | Task 1 |
| launchctl without -w in reload | Task 12 |
| SUDO_USER for system scope status | Task 6 |
