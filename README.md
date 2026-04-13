# lagent

A `systemctl`-style CLI for managing macOS LaunchAgents.

## Installation

```bash
just install
```

This builds a release binary and copies it to `/usr/local/bin/lagent`.

**Requirements:** macOS, Rust toolchain (for building), `just` (for the recipes).

## Usage

```
lagent <command> [--user]
```

The `--user` flag targets `~/Library/LaunchAgents`. Without it, commands operate on `/Library/LaunchAgents` and require `sudo`.

### Commands

| Command | Description |
|---|---|
| `list` | List all agents with loaded/running status |
| `status <agent>` | Show status for a single agent |
| `enable <agent>` | Load and mark the agent to start at login |
| `disable <agent>` | Unload and prevent the agent from starting at login |
| `start <agent>` | Start a loaded agent immediately |
| `stop <agent>` | Stop a running agent |
| `restart <agent>` | Stop then start an agent |
| `reload <agent>` | Reload plist from disk, skipping if unchanged |
| `validate <agent>` | Check plist syntax and schema |
| `completions <shell>` | Print shell completion script |

### Agent names

Commands that take `<agent>` accept either the plist filename (with or without `.plist`) or the `Label` value inside the plist.

```bash
lagent status com.example.myapp
lagent status com.example.myapp.plist
```

### Examples

```bash
# List all user agents
lagent list --user

# Enable a user agent
lagent enable --user com.example.myapp

# Reload after editing a plist (skips restart if file unchanged)
lagent reload --user com.example.myapp

# Validate a plist before loading
lagent validate --user com.example.myapp

# Install zsh completions
lagent completions zsh > ~/.zfunc/_lagent
```

## Smart reload

`reload` computes a SHA256 hash of the plist file and compares it to the last known hash. If the file hasn't changed, it skips the unload/load cycle entirely. If it has changed and the agent was running, it restarts automatically after reloading.

## Shell completions

```bash
# zsh
lagent completions zsh > ~/.zfunc/_lagent

# bash
lagent completions bash > /usr/local/etc/bash_completion.d/lagent

# fish
lagent completions fish > ~/.config/fish/completions/lagent.fish
```

## Development

```bash
just test     # run tests
just build    # build release binary
just install  # install to /usr/local/bin
just clean    # clean build artifacts
```

## License

MIT
