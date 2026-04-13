# lagent Design Spec

**Date:** 2026-04-13  
**Status:** Approved

## Overview

`lagent` is a Rust CLI tool for managing macOS LaunchAgents, modeled on `systemctl`. It provides a consistent, ergonomic interface for the most common launchd operations, targeting both user-level and system-level agents.

---

## Goals

- Single compiled binary, easy to install (`make install` or drop into `/usr/local/bin`)
- Ergonomic: accept agents by label or filename
- Safe: fail gracefully when root is required but not present
- Smart `reload`: avoid unnecessary restarts when plist hasn't changed
- Shell completion out of the box

---

## Non-Goals

- Managing `LaunchDaemons` (system daemons in `/Library/LaunchDaemons`)
- Managing per-session agents (`~/Library/LaunchAgents` variants under `loginwindow`)
- GUI or TUI interface
- Watching/monitoring agents over time

---

## Technology

| Concern | Choice |
|---|---|
| Language | Rust (stable) |
| CLI parsing | `clap` v4 (derive macros) |
| Plist parsing | `plist` crate |
| Hashing | `sha2` crate |
| Error handling | `anyhow` |
| Terminal output | `owo-colors` |
| Shell completions | `clap_complete` |

---

## Scope & Permissions

**Default scope:** `/Library/LaunchAgents` (system-level, requires root)

**`--user` flag:** `~/Library/LaunchAgents` (user-level, no root needed)

If a system-scope command is run without root, `lagent` exits immediately with a clear message:

```
error: this operation requires root. Re-run with sudo.
```

---

## Agent Resolution

Commands that take an `<agent>` argument resolve it as follows:

1. Strip `.plist` suffix if present
2. Scan the target directory for a `.plist` file whose **filename** (without `.plist`) matches, OR whose **`Label` key** value matches
3. If zero matches: exit with `error: no agent found matching '<input>'`
4. If multiple matches: exit with a list of matches and `error: ambiguous agent name, be more specific`

Resolution is case-sensitive.

---

## Commands

### `lagent list [--user]`

Scans the target LaunchAgents directory and cross-references with `launchctl list` output to determine state. All `.plist` files in the directory are shown regardless of loaded state.

Output columns:
- `LABEL` — from the plist `Label` key (or filename if unreadable)
- `LOADED` — whether launchctl has the agent registered (`yes`/`no`)
- `RUNNING` — whether a process is currently running (`yes`/`no`)
- `PID` — process ID if running, `-` otherwise

Example:
```
LABEL                          LOADED   RUNNING  PID
com.example.myapp              yes      yes      1234
com.example.otheragent         yes      no       -
com.example.disabled           no       no       -
```

### `lagent status <agent> [--user]`

Same as `list` but for a single resolved agent. Exits with code 1 if the agent is not loaded.

### `lagent enable <agent> [--user]`

Loads and marks the agent to start at login:
```
launchctl load -w <path>
```
Also writes a SHA256 hash of the plist to `~/.local/share/lagent/<label>.hash`.

### `lagent disable <agent> [--user]`

Unloads and prevents the agent from starting at login:
```
launchctl unload -w <path>
```
Clears the stored hash file.

### `lagent start <agent> [--user]`

Starts a loaded agent immediately:
```
launchctl start <label>
```

### `lagent stop <agent> [--user]`

Stops a running agent immediately:
```
launchctl stop <label>
```

### `lagent restart <agent> [--user]`

Stops then starts the agent. Does not reload the plist definition — use `reload` for that.

### `lagent reload <agent> [--user]`

Re-reads the plist from disk and applies changes without unnecessary restarts:

1. Compute SHA256 of current plist file on disk
2. Compare to stored hash in `~/.local/share/lagent/<label>.hash`
3. If **unchanged**: print `No changes detected in <label>, skipping.` and exit 0
4. If **changed** (or no stored hash):
   a. Note whether the agent is currently running
   b. `launchctl unload <path>` (without `-w` — preserves existing disabled/enabled state)
   c. `launchctl load <path>` (without `-w`)
   d. If it was running: `launchctl start <label>`
   e. Write updated hash to state file

### `lagent validate <agent> [--user]`

Two-stage validation:

1. **Syntax** — run `plutil -lint <path>` (macOS built-in). Fails fast if invalid plist.
2. **Schema** — parse with the `plist` crate and check:
   - `Label` key exists and is a non-empty string
   - Either `Program` or `ProgramArguments` exists (not both missing)
   - Warn (non-fatal) on unknown top-level keys not in the standard launchd key set

Exits 0 if valid, 1 if any error, prints warnings to stderr.

### `lagent completions <shell>`

Prints shell completion script to stdout. Supported shells: `bash`, `zsh`, `fish`, `elvish`.

Install example:
```bash
lagent completions zsh > ~/.zfunc/_lagent
```

---

## State Storage

Hash files live at: `~/.local/share/lagent/<label>.hash`

- Created/updated by: `enable`, `reload` (on change)
- Deleted by: `disable`
- Format: hex-encoded SHA256 of the raw plist file bytes

If the hash file is missing when `reload` is called, treat as changed and proceed with unload+load.

---

## Error Handling

- All errors use `anyhow` for chaining context
- User-facing errors are printed to stderr in plain English, not Rust debug format
- `launchctl` subprocess failures surface the exit code and any stderr output
- `plutil` failures surface the raw error message from the tool

---

## Distribution

- Build: `cargo build --release` → single binary at `target/release/lagent`
- `Makefile` with an `install` target that copies to `/usr/local/bin/lagent`
- No runtime dependencies beyond macOS system tools (`launchctl`, `plutil`)

---

## Project Structure

```
lagent/
├── Cargo.toml
├── Makefile
├── src/
│   ├── main.rs          # clap entrypoint, command dispatch
│   ├── cli.rs           # clap derive structs
│   ├── agent.rs         # agent resolution logic
│   ├── launchctl.rs     # subprocess wrappers for launchctl
│   ├── scope.rs         # scope/path resolution (system vs user)
│   ├── state.rs         # hash storage read/write
│   └── validate.rs      # plutil + plist schema validation
└── docs/
    └── superpowers/
        └── specs/
            └── 2026-04-13-lagent-design.md
```
