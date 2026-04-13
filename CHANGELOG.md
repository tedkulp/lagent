# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-13

### Added
- `list` command — show all agents in the target directory with loaded/running/PID status
- `status` command — show status for a single resolved agent; exits 1 if not loaded
- `enable` command — load agent with `launchctl load -w` and store plist hash
- `disable` command — unload agent with `launchctl unload -w` and clear stored hash
- `start` command — start a loaded agent immediately
- `stop` command — stop a running agent
- `restart` command — stop then start an agent
- `reload` command — reload plist from disk, skipping unload/load if file is unchanged (SHA256 comparison); restarts automatically if agent was running
- `validate` command — two-stage plist validation: `plutil -lint` syntax check, then schema check via the `plist` crate (requires `Label`, requires `Program` or `ProgramArguments`, warns on unknown keys)
- `completions` command — generate shell completion scripts for bash, zsh, fish, and elvish
- `--user` flag on all commands to target `~/Library/LaunchAgents` instead of `/Library/LaunchAgents`
- Agent resolution by filename stem or `Label` key value (case-sensitive)
- Root check for system-scope commands with a clear error message
- SHA256 hash state stored at `~/.local/share/lagent/<label>.hash`
- Makefile replaced with `justfile` (recipes: `test`, `build`, `install`, `clean`)

[Unreleased]: https://github.com/tedkulp/lagent/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tedkulp/lagent/releases/tag/v0.1.0
