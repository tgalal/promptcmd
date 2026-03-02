# CHANGELOG

## Version 1.1.0-beta.5 (2026-03-02)

### Fixed

- fifo as default channel instead of nc
- promptctl config edit not working if config does not exist yet

## Version 1.1.0-beta.4 (2026-03-01)

### Fixed

- Wrong exported functions name in bash when auto shell

## Version 1.1.0-beta.3 (2026-02-28)

### Changed

- Disable ssh integration in windows

## Version 1.1.0-beta2 (2026-02-27)

### Changed

- Remove unused openssh

## Version 1.1.0-beta (2026-02-27)

### Added

- Remote SSH Integration

### Fixed

- Path traversal in prompt names

## Version 1.0.7 (2026-01-29)

### Fixed

- Command-given model resolution not properly overriding FM
- Variant name resolution

## Version 1.0.6 (2026-01-22)

### Added

- Support config_ttl in frontmatter config

## Version 1.0.5 (2026-01-22)

### Fixed

- Inconsistent terminating newline in output

## Version 1.0.4 (2026-01-22)

### Added

- Fenced code extraction by setting output format to code in frontmatter

## Version 1.0.3 (2026-01-20)

### Added

- Render only mode for basic integration with other tools
- Support for streaming output
- Ask Helper for interactive inputs

### Fixed

- Shortform of openai in frontmatter

## Version 1.0.2 (2026-01-19)

### Added

- Configurable Endpoint for OpenAI
- Configuration via Environment Variables
- Ask Helper for interactive inputs

### Fixed

- Handling of model in frontmatter when given by provider name only

## Version 1.0.1 (2026-01-14)

### Changed

- Creating a prompt enables it by default
- Replace `--enable` with `--no-enable` during creation

## Version 1.0.0 (2026-01-14)

### Added

- Shebang support
- Caching via cache_ttl config property

### Changed

- Moved installation directory to $HOME/.promptcmd/bin/
- Moved db storage to $HOME/.promptcmd/bin/
- Moved prompt storage to $CONFIG/promptcmd/prompts/

### Fixed

- Handling integers in output schema
- Configuration under [providers] was ignored

## Version 0.7.1 (2026-01-12)

### Fixed

- Typos in help messages

## Version 0.7.0 (2026-01-12)

### Added

- concat helper for concatenating strings in a template
- exec helper for executing commands and rendering output in templates
- prompt helper for executing other prompts
- enum support for inputs

### Fixed

- Now types other that stringed are passed properly to handlebars
- Concurrent access to the sqlite database

## Version 0.6.1 (2026-01-08)

### Added

- "i" as alias to import command
- Allow overriding model during import

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

