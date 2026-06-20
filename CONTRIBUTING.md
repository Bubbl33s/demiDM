# Contributing to DemiDM

Thank you for your interest in contributing to DemiDM! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates. When creating a bug report, include:

- Clear descriptive title
- Steps to reproduce the issue
- Expected vs actual behavior
- System information (kernel version, distro, Rust version)
- Relevant logs from `/tmp/demidm.log`

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md).

### Suggesting Features

Feature suggestions are welcome! Please:

- Check existing issues for similar requests
- Explain the use case and benefits
- Consider implementation complexity

Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md).

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run the verification suite:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   cargo build --release
   ```
5. Commit with conventional commit messages:
   ```
   feat(auth): add biometric authentication support
   fix(render): correct widget positioning on resize
   docs(lua): update widget API documentation
   ```
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request

Use the [PR template](.github/PULL_REQUEST_TEMPLATE.md).

## Development Setup

### Prerequisites

- Rust 1.85 or later
- Linux system with PAM development libraries
- pkg-config

### Building from Source

```bash
git clone https://github.com/Bubbl33s/demidm.git
cd demidm
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Style

- Follow `rustfmt` configuration (run `cargo fmt`)
- Address all `clippy` warnings (`cargo clippy -- -D warnings`)
- Write documentation for public APIs
- Add tests for new functionality

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Scopes:** `auth`, `render`, `lua`, `input`, `graphics`, `state`, `widget`, `tty`, `events`

**Examples:**
```
feat(widget): add clock widget with configurable format
fix(auth): handle PAM conversation errors gracefully
docs(readme): add installation instructions for Arch Linux
```

## Architecture Guidelines

### Event-Driven Design

All state mutations flow through the event bus:

```
Input/Lua/PAM → AppEvent → apply_event() → AppState → Renderer
```

- Never mutate `AppState` directly outside `apply_event()`
- Never use `Arc<Mutex<AppState>>` for shared state
- Main thread owns the TTY and renderer exclusively

### Security

- Passwords must use `secrecy::SecretString` with `zeroize()`
- PAM authentication runs in isolated worker threads
- Never log sensitive information
- Follow the principle of least privilege

### File Organization

- Max ~400-500 lines per file
- Single responsibility per module
- Related functionality grouped together
- Public APIs via `pub use` re-exports

## Testing

- Write unit tests for pure functions
- Write integration tests for complex workflows
- Test error paths and edge cases
- Ensure tests pass on CI before merging

## Documentation

- Document all public functions with `///` doc comments
- Include examples in doc comments where helpful
- Keep README and docs up to date
- Update CHANGELOG.md for user-facing changes

## Questions?

Feel free to open an issue for questions or join the discussion!
