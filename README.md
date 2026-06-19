# DemiDM

A lightweight, scriptable display manager for Linux written in Rust.

DemiDM provides a fast, secure, and customizable login experience with a TUI interface powered by [ratatui](https://github.com/ratatui/ratatui), PAM authentication, and Lua scripting for configuration and widgets.

![DemiDM Screenshot](docs/images/screenshot.png)

## Features

- **Fast & Lightweight** - Minimal resource footprint, written in Rust
- **Secure Authentication** - PAM-based with zeroize-on-drop password handling
- **Lua Scripting** - Fully configurable via Lua scripts (themes, widgets, hooks)
- **Multiple Graphics Backends** - Framebuffer, Kitty protocol, Überzug++, ASCII art
- **TUI Interface** - Terminal-based login with customizable layouts
- **Widget System** - Dynamic widgets powered by Lua
- **Event-Driven Architecture** - Async tokio-based event loop

## Requirements

- Linux kernel with framebuffer support
- PAM (Pluggable Authentication Modules)
- Rust 1.75+ (for building from source)

### Optional Dependencies

- `ueberzugpp` - For image display via Überzug++
- A terminal supporting Kitty graphics protocol (optional)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/Bubbl33s/demidm.git
cd demidm

# Build in release mode
cargo build --release

# Install the binary
sudo cp target/release/demidm /usr/bin/

# Install PAM configuration
sudo cp pam/demidm.pam /etc/pam.d/demidm
```

### Systemd Service

Create `/etc/systemd/system/demidm.service`:

```ini
[Unit]
Description=DemiDM Display Manager
After=systemd-user-sessions.service getty@tty1.service
Conflicts=getty@tty1.service

[Service]
ExecStart=/usr/bin/demidm --tty /dev/tty1
Restart=always
RestartSec=3

[Install]
WantedBy=graphical.target
```

Enable and start:

```bash
sudo systemctl enable demidm
sudo systemctl start demidm
```

## Configuration

DemiDM uses Lua for configuration. The config file is searched in the following order:

1. `$DEMIDM_CONFIG_DIR/init.lua`
2. `$XDG_CONFIG_HOME/demidm/init.lua`
3. `/etc/demidm/init.lua`

### Example Configuration

```lua
-- /etc/demidm/init.lua

-- Theme configuration
demidm.theme({
    background = "#1a1b26",
    foreground = "#c0caf5",
    accent = "#7aa2f7",
    error = "#f7768e",
})

-- Login box appearance
demidm.login_box({
    title = "Welcome",
    username_placeholder = "Username",
    password_placeholder = "Password",
    width = 40,
    height = 12,
})

-- Register a widget
demidm.widget({
    id = "clock",
    position = { x = 1, y = 1 },
    render = function(ctx)
        local time = os.date("%H:%M:%S")
        ctx:draw_text(0, 0, time, "accent")
    end
})

-- Hooks
demidm.on("login_success", function(username)
    demidm.log("User logged in: " .. username)
end)
```

## Lua API Reference

### Core Functions

| Function | Description |
|----------|-------------|
| `demidm.theme(config)` | Set theme colors |
| `demidm.login_box(config)` | Configure login box appearance |
| `demidm.widget(def)` | Register a widget |
| `demidm.log(message)` | Log a message |
| `demidm.on(event, callback)` | Register an event hook |

### Widget API

Widgets can use the following context methods in their `render` function:

| Method | Description |
|--------|-------------|
| `ctx:draw_text(x, y, text, style)` | Draw text at position |
| `ctx:draw_rect(x, y, w, h, style)` | Draw a rectangle |
| `ctx:width()` | Get widget width |
| `ctx:height()` | Get widget height |

### Events

| Event | Payload | Description |
|-------|---------|-------------|
| `login_success` | `username` | User authenticated successfully |
| `login_failed` | `username` | Authentication failed |
| `session_start` | `username` | Desktop session starting |
| `shutdown` | - | Display manager shutting down |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Main Thread                           │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │ Input Poller │  │ Event Loop   │  │ Renderer (ratatui)  │ │
│  └──────┬──────┘  └──────┬───────┘  └─────────────────────┘ │
│         │                │                                    │
│         └────────────────┼────────────────────────────────────│
│                          │                                    │
│  ┌───────────────────────┼───────────────────────────────────┐│
│  │                  AppState                                  ││
│  └───────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
    ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
    │   PAM   │         │   Lua   │         │ Widgets │
    │ Worker  │         │ Runtime │         │ Runner  │
    └─────────┘         └─────────┘         └─────────┘
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

### Project Structure

```
src/
├── auth/           # PAM authentication and session management
├── graphics/       # Graphics backends (framebuffer, kitty, etc.)
├── input/          # Input capture and keybinds
├── lua_runtime/    # Lua scripting engine and API
├── renderer/       # TUI rendering with ratatui
├── state/          # Application state management
├── widget/         # Widget definitions
├── widget_runner/  # Widget execution engine
├── errors.rs       # Error types
├── events.rs       # Event bus
├── main.rs         # Entry point
└── tty.rs          # TTY management
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ratatui](https://github.com/ratatui/ratatui) - Terminal UI library
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [mlua](https://github.com/khvzak/mlua) - Lua bindings for Rust
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
