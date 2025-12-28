# TNJ - Tasks, Notes, Journal

A lightweight, terminal-based application for managing tasks, notes, and journal entries. Built with Rust and featuring an intuitive TUI (Text User Interface), TNJ helps you stay organized without leaving your terminal.

## Features

- **Task Management**: Create, organize, and track tasks with due dates, status, and tags
- **Note Taking**: Capture and organize notes with rich content and tagging
- **Journal Entries**: Maintain a daily journal with date-based organization
- **Tagging System**: Organize items with tags and filter by them
- **Notebooks**: Group related tasks, notes, and journal entries into notebooks
- **Advanced Filtering**: Filter by tags, status, archive state, and more
- **SQLite Database**: All data stored locally in a SQLite database
- **Keyboard-Driven**: Fully keyboard-navigable TUI interface
- **CLI Commands**: Quick commands to add items without opening the TUI
- **Dev/Prod Profiles**: Separate development and production environments

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/tnj.git
cd tnj

# Build the project
cargo build --release

# The binary will be in target/release/tnj
```

### Install with Cargo

```bash
cargo install --path .
```

## Usage

### Interactive TUI Mode

Launch the interactive terminal user interface:

```bash
tnj tui
# or simply
tnj
```

### CLI Commands

#### Add a Task

```bash
tnj add-task "Complete project documentation" --due 2024-12-31 --tags "work,important"
```

#### Add a Note

```bash
tnj add-note "Meeting Notes" --content "Discussed project timeline..." --tags "meeting,work"
```

#### Add a Journal Entry

```bash
tnj add-journal "Today I worked on the new feature..." --title "Daily Reflection" --tags "personal"
```

### Development Mode

Use development mode to work with a separate database and configuration:

```bash
tnj --dev tui
```

## Configuration

TNJ uses configuration files stored in your system's configuration directory:
- **Linux**: `~/.config/tnj/`
- **macOS**: `~/Library/Application Support/tnj/`
- **Windows**: `%APPDATA%\tnj\`

The configuration file (`config.toml`) is automatically created on first run.

## Keyboard Shortcuts

### General
- `q` or `Esc`: Quit/Close
- `?`: Show help
- `Tab`: Switch between tabs (Tasks, Notes, Journal)
- `Ctrl+S`: Save
- `Ctrl+C`: Copy to clipboard

### Navigation
- `j` / `↓`: Move down
- `k` / `↑`: Move up
- `Enter`: Open/Edit item
- `n`: New item
- `d`: Delete item
- `a`: Archive/Unarchive item

### Filtering
- `f`: Open filter modal
- `Ctrl+F`: Toggle filter sidebar

*Note: Full keyboard shortcuts are available in the help menu (press `?` in the TUI)*

## Project Structure

```
tnj/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library root
│   ├── cli.rs           # CLI command handling
│   ├── config.rs        # Configuration management
│   ├── database.rs      # SQLite database operations
│   ├── models.rs        # Data models (Task, Note, JournalEntry, Notebook)
│   ├── utils.rs         # Utility functions
│   └── tui/             # TUI components
│       ├── app.rs       # Main application state
│       ├── events.rs    # Event handling
│       ├── layout.rs    # Layout management
│       ├── render.rs    # Rendering logic
│       └── widgets/     # UI widgets
├── Cargo.toml           # Rust project configuration
└── README.md            # This file
```

## Development

### Running in Development Mode

```bash
# Run with development profile
cargo run -- --dev tui

# Or build and run
cargo build
./target/debug/tnj --dev tui
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Contributing

Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI library
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite database driver
- [chrono](https://github.com/chronotope/chrono) - Date and time handling
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [termimad](https://github.com/Canop/termimad) - Markdown rendering in terminal

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
