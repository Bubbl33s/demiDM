# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- Event-driven architecture with tokio
- PAM authentication with secure password handling
- TUI login interface with ratatui
- Lua scripting runtime for configuration
- Widget system with dynamic rendering
- Multiple graphics backends (framebuffer, kitty, ueberzugpp, ASCII art)
- Input capture and keybind system
- TTY management with panic recovery
- Theme and login box configuration API
- Hook system for lifecycle events
- Image scaling utilities
- Session launch with privilege dropping

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- Password zeroization with secrecy crate
- PAM worker isolation
- Session privilege dropping via setuid/setgid

---

## [0.1.0] - 2026-06-12

### Added
- Initial release
- Core display manager functionality
- Basic PAM authentication
- TUI rendering with ratatui
- Lua configuration support
- Widget framework
- Graphics backend trait and implementations

[Unreleased]: https://github.com/Bubbl33s/demidm/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Bubbl33s/demidm/releases/tag/v0.1.0
