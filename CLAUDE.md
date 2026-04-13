# lagent

A `systemctl`-style CLI for managing macOS LaunchAgents. Single Rust binary.

## Project layout

```
src/
  main.rs        # clap entrypoint + command dispatch
  cli.rs         # clap derive structs (Commands enum, --user flag)
  commands.rs    # all command implementations
  agent.rs       # resolve agent by label or filename stem
  launchctl.rs   # subprocess wrappers for launchctl
  scope.rs       # target directory + root check
  state.rs       # SHA256 hash read/write/delete (~/.local/share/lagent/)
  validate.rs    # plutil syntax check + plist crate schema check
```

## Common tasks

```bash
just test       # cargo test
just build      # cargo build --release
just install    # install binary to /usr/local/bin/lagent
just clean      # cargo clean
```

## Key design decisions

- **System scope** (`/Library/LaunchAgents`) is the default; requires `sudo`. `--user` targets `~/Library/LaunchAgents`.
- **Agent resolution**: accepts label (`com.example.foo`) or filename stem (`com.example.foo` or `com.example.foo.plist`). Matches by filename stem first, then by `Label` key in the plist. Errors on zero or multiple matches.
- **`reload` skips unnecessary restarts**: computes SHA256 of the plist file, compares to `~/.local/share/lagent/<label>.hash`. Skips unload/load if unchanged. Restarts only if the agent was running before reload.
- **`reload` vs `enable/disable`**: `reload` uses `launchctl load/unload` without `-w` (preserves the existing enabled/disabled state). `enable`/`disable` use `-w`.
- **System scope + sudo**: when running as root for system-scope commands, `launchctl list` is invoked as `$SUDO_USER` to query the user's GUI launchd session (where LaunchAgents actually live).
- **Exit codes**: commands that signal semantic failure (e.g. `status` when not loaded, `validate` on errors) call `std::process::exit(1)` directly after printing to stderr, rather than propagating an error through `main`. This avoids a double-printed error line.

## State files

Hash files: `~/.local/share/lagent/<label>.hash`
- Written by: `enable`, `reload` (on change)
- Deleted by: `disable`
- Overridden in tests via `LAGENT_STATE_DIR` env var

## Testing

- Tests use `tempfile` crate for isolated directories
- `state.rs` tests use a `static Mutex` to serialize env var mutation (`LAGENT_STATE_DIR`)
- `launchctl.rs` has no unit tests (wraps OS commands); coverage comes from integration/smoke tests
- Run: `cargo test`; 19 tests expected

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` v4 (derive) | CLI parsing |
| `clap_complete` | Shell completion generation |
| `plist` | Parse `.plist` files for agent resolution and validation |
| `sha2` 0.10 | SHA256 hashing for reload change detection |
| `anyhow` | Error handling with context chaining |
| `owo-colors` | Colored terminal output |
| `tempfile` (dev) | Temp directories in tests |

## Release process

1. Add all changes to `CHANGELOG.md` under `[Unreleased]`
2. Move `[Unreleased]` entries to a new `[x.y.z] - YYYY-MM-DD` section
3. Update the comparison links at the bottom of `CHANGELOG.md`
4. Bump `version` in `Cargo.toml`
5. `just build` — confirm clean build with no warnings
6. `just test` — confirm all tests pass
7. Commit: `git commit -m "chore: release vx.y.z"`
8. Tag: `git tag vx.y.z`
