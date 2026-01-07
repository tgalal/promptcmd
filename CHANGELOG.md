# CHANGELOG

## Version 0.6.0 (2026-01-07)

### Changed

- Print actual config parsing error
- Change ctl config to be a subcommand itself
- Add success status to stats --last

### Fixed

- Fix google and openrouter not handled as variants

### Removed

- promptctl list --config

## Version 0.5.0 (2026-01-06)

### Added

- Windows support

## Version 0.4.10 (2026-01-06)

### Changed

- Install using hardlinks on Windows.

## Version 0.4.9 (2026-01-06)

### Added

- New "config" ctl command for editing the config.toml

### Changed

- Add onboarding hints about the config command

### Removed

- `list --config` in favor of `config --list`

### Fixed

- Fixed homebrew tap repo name

