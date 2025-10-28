# Hyprland Keybinding Manager

> A secure, professional-grade GTK4 GUI application for managing Hyprland keybindings with real-time conflict detection and automatic backup system.

[![Version](https://img.shields.io/badge/version-1.0.7-blue.svg)](https://github.com/tidynest/hypr-keybind-manager/releases)
[![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4.0-blue.svg)](https://www.gtk.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-166%20passing-brightgreen.svg)](#testing)

---

## Table of Contents

- [Overview](#overview)
  - [Key Features](#key-features)
- [Screenshots](#screenshots)
- [Architecture](#architecture)
  - [High-Level Overview](#high-level-overview)
  - [Security Architecture](#security-architecture)
- [Installation](#installation)
  - [Prerequisites](#prerequisites)
  - [Building from Source](#building-from-source)
  - [Distribution Packages](#distribution-packages)
- [Usage](#usage)
  - [Quick Start](#quick-start)
  - [Command-Line Interface](#command-line-interface)
  - [Workflow](#workflow)
- [Features in Detail](#features-in-detail)
  - [Conflict Detection](#conflict-detection)
  - [Security Validation](#security-validation)
  - [Backup System](#backup-system)
- [Development](#development)
  - [Project Structure](#project-structure)
  - [Testing](#testing)
  - [Code Quality Standards](#code-quality-standards)
- [Technical Highlights](#technical-highlights)
  - [Design Patterns Demonstrated](#design-patterns-demonstrated)
  - [Performance Optimisations](#performance-optimisations)
  - [Security Considerations](#security-considerations)
  - [Modern Rust Practices](#modern-rust-practices)
- [Roadmap](#roadmap)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)
- [Contact](#contact)
- [Project Stats](#project-stats)

---

## Overview

**Hyprland Keybinding Manager** [*hypr-keybind-manager*] is a desktop application that provides a graphical interface for managing [Hyprland](https://hypr.land/) keybindings. Built with [Rust](https://www.rust-lang.org/) and [GTK4](https://www.gtk.org/), it offers a robust, secure, and user-friendly way to create, edit, and organise your keybindings while preventing conflicts and protecting against malicious configurations.

### Key Features

- **Full [CRUD](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) Operations**: Create, read, update, and delete keybindings through an intuitive GUI
- **Real-Time Conflict Detection**: Instantly identifies duplicate key combinations with [O(1)](https://en.wikipedia.org/wiki/Time_complexity#Constant_time) performance
- **Three-Layer Security Validation**: Prevents [shell injection](https://en.wikipedia.org/wiki/Code_injection#Shell_injection), dangerous commands, and encoded payloads
- **Automatic Backup System**: Every change creates timestamped backups with [atomic write operations](https://en.wikipedia.org/wiki/Atomicity_(database_systems))
- **Search & Filter**: Real-time search across key combinations, dispatchers, and arguments
- **Keyboard Navigation**: Arrow keys, Enter, and Tab for efficient workflow
- **Modern GTK4 UI**: Clean, responsive interface with the intention of following [GNOME HIG](https://developer.gnome.org/hig/) guidelines

This project is likely to be enhanced and further developed in the near future.

---

## Screenshots

> **Note**: Screenshots coming soon. The application features a clean, modern interface with:
> - Left panel: Searchable keybinding list with "Add Keybinding" and "Manage Backups" buttons
> - Right panel: Detailed view of selected keybinding with Edit/Delete buttons
> - Top panel: Conflict warnings (when detected)

### Placeholder Images
```
**[Main Window]**        [Edit Dialog]        [Backup Management]
┌──────────────────┐    ┌──────────────┐     ┌───────────────────┐
│ Search: [____]   │    │ Edit Binding │     │ Manage Backups    │
│  + Add Binding   │    │              │     │                   │
│  <=> Backups     │    │ Key: [____]  │     │ 2025-01-15 14:30  │
│                  │    │ Disp: [___]  │     │ 2025-01-14 09:15  │
│  SUPER+K → exec  │    │ Args: [___]  │     │ 2025-01-13 18:45  │
│  SUPER+M → exec  │    │ Type: [___]  │     │                   │
│  SUPER+Q → kill  │    │              │     │ [Restore][Delete] │
└──────────────────┘    │ [Save][Canc] │     └───────────────────┘
                        └──────────────┘
```

---

## Architecture

### High-Level Overview

```mermaid
graph TB
    User[User Interface<br/>GTK4 Components]
    Controller[Controller<br/>MVC Mediator]
    ConfigMgr[ConfigManager<br/>File I/O + Backups]
    Parser[Parser<br/>Nom Combinators]
    Conflict[ConflictDetector<br/>HashMap O1]
    Validator[Security Validator<br/>3 Layers]

    User -->|User Actions| Controller
    Controller -->|Load/Save| ConfigMgr
    ConfigMgr -->|Parse| Parser
    Parser -->|Keybindings| Controller
    Controller -->|Detect| Conflict
    Controller -->|Validate| Validator
    Validator -->|Block/Warn| Controller
    Controller -->|Update UI| User

    style User fill:#e1f5ff,color:#000
    style Controller fill:#fff4e1,color:#000
    style Validator fill:#ffe1e1,color:#000
```

<details>
<summary>ASCII Fallback Diagram</summary>

```
┌──────────────────────────────────────────────────────────┐
│                  User Interface (GTK4)                   │
│----------------------------------------------------------│
│   ┌──────────────┐  ┌───────────────┐  ┌─────────────┐   │
│   │ Keybind List │  │ Details Panel │  │ Edit Dialog │   │
│   └──────────────┘  └───────────────┘  └─────────────┘   │
└─────────────────────────────┬────────────────────────────┘
                              │
                              ↓
                 ┌──────────────────────────┐
                 │      Controller (MVC)    │
                 │--------------------------│
                 │  - load_keybindings()    │
                 │  - filter_keybindings()  │
                 │  - delete_keybinding()   │
                 └───────┬──────────┬───────┘
                         │          │
           ┌─────────────┘          └───────────────┐
           ↓                                        ↓
 ┌────────────────────┐                  ┌────────────────────┐
 │   ConfigManager    │                  │   ConflictDetect   │ 
 │--------------------│                  │--------------------│
 │  - read_config()   │                  │  - HashMap O(1)    │
 │  - write_backup()  │                  │  - find_conflicts  │
 │  - atomic_write()  │                  └────────────────────┘
 └─────────┬──────────┘
           │
           ↓
┌──────────────────────┐
│  Security Validator  │
│----------------------│
│  Layer 1: Injection  │
│  Layer 2: Danger     │
│  Layer 3: Config     │
└──────────────────────┘
```
</details>

### Security Architecture

The application implements **[defence in depth](https://en.wikipedia.org/wiki/Defense_in_depth_(computing))** with three independent validation layers:

```mermaid
graph LR
    Input[User Input] --> L1[Layer 1<br/>Injection Prevention]
    L1 --> L2[Layer 2<br/>Danger Detection]
    L2 --> L3[Layer 3<br/>Config Validation]
    L3 --> Accept[✅ Accept]
    L3 --> Reject[❌ Reject]

L1 -.->|Blocks| Inject[Shell Metacharacters<br/>$, ;, pipe, &, backticks]
    L2 -.->|Detects| Danger[Critical Patterns<br/>rm -rf /, dd, fork bombs]
    L3 -.->|Validates| Config[Dispatcher Whitelist<br/>Argument Length<br/>Encoding Detection]

    style L1 fill:#ffe1e1,color:#000
    style L2 fill:#fff4e1,color:#000
    style L3 fill:#e1ffe1,color:#000
    style Accept fill:#d4f1d4,color:#000
    style Reject fill:#ffd4d4,color:#000
```

For detailed security information, see [SECURITY.md](SECURITY.md).

---

## Installation

### Prerequisites

- **Rust 1.83+** (with [Cargo](https://doc.rust-lang.org/cargo/))
- **GTK4 4.0+** development libraries
- **Hyprland** [Wayland](https://wayland.freedesktop.org/) compositor (for runtime use)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/tidynest/hypr-keybind-manager.git
cd hypr-keybind-manager

# Build release binary
cargo build --release

# Install to system (optional)
sudo cp target/release/hypr-keybind-manager /usr/local/bin/
```

### Development Workflow

The project includes automatic output filtering for clean terminal logging:

```bash
# Run during development (GTK warnings automatically filtered)
cargo run -- gui -c /tmp/hyprland-test.conf

# User action logs appear in real-time with emoji indicators:
→ Launching GUI...
👆 Selected: SUPER+SHIFT+W
✏️  Edit button clicked for: SUPER+SHIFT+W
✅ Keybinding updated successfully
```

The `.cargo/config.toml` configures a custom runner that:
- Filters GTK warnings (`Gtk-WARNING`, `GtkGizmo`, `Unknown key`)
- Preserves application logs (`eprintln!` with emoji indicators)
- Removes blank lines for cleaner output
- Uses `grep --line-buffered` for real-time log display

All user actions are logged to stderr for debugging and visibility.

### Distribution Packages

> **Future Packages**: AUR package, Nix flake, and prebuilt binaries for Arch, Fedora, and Ubuntu.

---

## Usage

### Quick Start

```bash
# Launch GUI with default Hyprland config
hypr-keybind-manager gui

# Use specific config file
hypr-keybind-manager gui -c ~/.config/hypr/hyprland.conf

# Check for conflicts (CLI - no GUI)
hypr-keybind-manager check

# List all keybindings (CLI - no GUI)
hypr-keybind-manager list

# Test with sample config (won't modify your real config)
hypr-keybind-manager gui -c /tmp/test-hyprland.conf
```

### Command-Line Interface

```bash
hypr-keybind-manager <COMMAND>

Commands:
  check       Check for keybinding conflicts
  list        List all keybindings
  gui         Launch GUI overlay
  help        Print this message or the help of the given subcommand(s)

Global Options:
  -h, --help     Print help
  -V, --version  Print version

Subcommand Options (available on check, list, and gui):
  -c, --config <FILE>  Path to Hyprland config file
                       [default: ~/.config/hypr/hyprland.conf]
```

### Workflow

1. **Launch the application**: Opens your Hyprland config
2. **Browse bindings**: Use search or scroll through the list
3. **Make changes**:
   - **Edit a binding**: Select → Click "Edit" → Modify → Save
   - **Add new binding**: Click "➕ Add Keybinding" → Fill form → Save
   - **Delete binding**: Select → Click "Delete" → Confirm
4. **Apply to Hyprland**: Click "Apply to Hyprland" button (header) → Hyprland reloads instantly
5. **Export/Import keybindings**:
   - **Export**: Menu → Export... → Choose file location → Saves all keybindings
   - **Import**: Menu → Import... → Choose mode (Replace or Merge) → Select file
6. **Manage backups**: Click "📦 Manage Backups" → Restore or delete backups

**Notes**:
- All changes are automatically backed up to `~/.config/hypr/backups/` with timestamps
- The UI automatically refreshes when the config file is modified externally (live file monitoring)

---

## Features in Detail

### Conflict Detection

The application uses a **[HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)-based conflict detector** with [O(1)](https://en.wikipedia.org/wiki/Time_complexity#Constant_time) average-case lookup performance:

- **Algorithm**: `HashMap<KeyCombo, Vec<Keybinding>>`
- **Normalisation**: Key combos are normalised (sorted modifiers, uppercase keys)
- **Real-Time**: Conflicts detected instantly as you type
- **Grouping**: All conflicting bindings displayed together

**Example**: If both `SUPER+K` and `SUPER+K` exist, the warning banner shows both with their actions.

### Security Validation

**Layer 1: Injection Prevention** (`core/validator.rs`)
- Whitelist-based dispatcher validation (only known-good dispatchers)
- Shell metacharacter detection (blocks `;`, `|`, `&`, `$`, backticks)
- Argument length limits (max 1000 characters)
- Key name validation (alphanumeric + safe special chars)

**Layer 2: Dangerous Command Detection** (`config/danger/`)
- Regex pattern matching for critical commands (`rm -rf /`, `dd if=/dev/zero of=/dev/sda`)
- HashSet lookup for dangerous executables (`sudo`, `chmod 777`, `mkfs`)
- Context-aware analysis (`chmod 644` flagged but explained)
- [Shannon entropy](https://en.wikipedia.org/wiki/Entropy_(information_theory)) detection for base64/hex encoded payloads (thresholds: 4.0 bits for base64, 3.0 bits for hex)

**Layer 3: Config Validation** (`config/validator.rs`)
- Coordinates Layers 1-2 and produces unified validation report
- Severity levels: Error (blocks), Warning (allows with notice), Info
- Transaction integration (validation before commit)

### Backup System

**Automatic Backups**:
- Created before every write operation
- Timestamped: `hyprland.conf.YYYY-MM-DD_HHMMSS`
- Stored in `~/.config/hypr/backups/`
- No user intervention required

**Atomic Writes**:
- Uses `atomic-write-file` crate (temp file + OS rename)
- Guarantees: Either old file intact OR new file complete
- Never corrupts config file, even on power loss

**Backup Management UI**:
- View all backups with formatted timestamps
- Restore any backup with one click
- Delete old backups to save space
- Safety backup created before restore

### Export/Import System

**Export Functionality**:
- Saves all current keybindings to a standalone `.conf` file
- Preserves exact Hyprland syntax (`bind = MODS, KEY, dispatcher, args`)
- Shareable across systems or users
- Useful for creating keybinding templates or presets

**Import Modes**:
- **Replace Mode**: Deletes all existing keybindings and replaces with imported ones
  - Use case: Completely switching to a new keybinding scheme
  - Warning dialog confirms before deletion
- **Merge Mode**: Keeps existing keybindings, adds imported ones
  - Skips duplicates (same key combo already exists)
  - Use case: Adding keybindings from multiple sources
  - Safe for incremental config building

**Conflict Handling**:
- Conflicts detected immediately after import
- Warning banner shows all duplicates
- User can resolve via Edit/Delete before applying to Hyprland

### Live Hyprland Integration

**Apply to Hyprland Button** (Header):
- Triggers `hyprctl reload` command via IPC
- Reloads Hyprland configuration instantly (no compositor restart)
- Changes take effect immediately in Hyprland
- Error feedback if `hyprctl` fails

**When to Use**:
- After making changes in the GUI (edit/add/delete)
- After importing keybindings
- After restoring from a backup
- **Note**: Changes are saved to config file immediately, but Hyprland only sees them after clicking "Apply to Hyprland"

### Live File Monitoring

**Automatic UI Refresh**:
- Uses Linux [inotify](https://man7.org/linux/man-pages/man7/inotify.7.html) to watch config file for modifications
- Zero CPU overhead when file unchanged
- Instant UI update when file modified externally (by editor, script, etc.)
- Prevents conflicts between GUI edits and manual edits

**How It Works**:
- Background file watcher (non-blocking)
- Detects `MODIFY` events on the config file
- Reloads and re-parses the entire config
- Updates all UI panels (keybind list, details, conflicts)

**Use Case**:
- Edit config in Neovim/VSCode while GUI is open
- Run scripts that modify `hyprland.conf`
- GUI always shows current state of the file

---

## Development

### Project Structure

```
hypr-keybind-manager/
├── README.md                      # Project overview and documentation hub (759 lines)
├── LICENSE                        # Apache 2.0 license (201 lines)
├── CONTRIBUTING.md                # Contribution guidelines (307 lines)
├── SECURITY.md                    # Security policy and threat model (484 lines)
├── Cargo.toml                     # Rust dependencies and metadata (58 lines)
├── build.rs                       # Build-time code generation (50 lines)
├── .cargo/                        # Project-specific cargo configuration
│   ├── config.toml                # Custom runner for filtered output (7 lines)
│   └── runner.sh                  # Output filter script (5 lines)
├── scripts/                       # Development and release scripts
│   ├── sync-version.sh            # Sync version numbers across docs (33 lines)
│   └── tag-release.sh             # Automated release tagging (79 lines)
├── docs/                          # Technical documentation
│   ├── ARCHITECTURE.md            # System design and data flow (762 lines)
│   ├── DESIGN_DECISIONS.md        # Rationale for architectural choices (1082 lines)
│   ├── ENTROPY_DETECTION.md       # Shannon entropy deep-dive (944 lines)
│   └── GTK_INSPECTOR_GUIDE.md     # GTK debugging guide (215 lines)
├── test-data/                     # Test configuration files
│   ├── hyprland-test.conf         # Safe test config (22 lines)
│   └── backups/                   # Test backup files
├── test-file-watcher.sh           # File watcher test script (315 lines)
└── src/                           # Source code (~5,500 lines total)
    ├── bin/                       # Binary utilities
    │   ├── measure_entropy.rs     # Entropy measurement tool (47 lines)
    │   └── test_manual.rs         # Manual testing utility (85 lines)
    ├── main.rs                    # CLI entry point (270 lines)
    ├── lib.rs                     # Library root (99 lines)
    ├── config/                    # Config file I/O (~1,970 lines)
    │   ├── mod.rs                 # ConfigManager (reads/writes with backups) (576 lines)
    │   ├── validator.rs           # Config validation (Layer 3) (293 lines)
    │   ├── danger/                # Dangerous command detection (Layer 2) (~935 lines)
    │   │   ├── mod.rs             # DangerDetector core (409 lines)
    │   │   ├── types.rs           # DangerLevel, DangerAssessment (41 lines)
    │   │   ├── patterns.rs        # Pattern builders (186 lines)
    │   │   ├── entropy.rs         # Shannon entropy detection (299 lines)
    │   │   └── tests/             # Modular test suite
    │   └── tests/                 # Config validation tests (164 lines)
    │       ├── mod.rs             # Test module organization (28 lines)
    │       └── validator_tests.rs # Validator unit tests (136 lines)
    ├── core/                      # Business logic (~1,295 lines)
    │   ├── types.rs               # Keybinding, KeyCombo, Modifier, BindType (209 lines)
    │   ├── parser.rs              # Parse Hyprland config syntax (nom) (371 lines)
    │   ├── conflict.rs            # ConflictDetector engine (HashMap) (105 lines)
    │   ├── validator.rs           # Injection prevention (Layer 1) (172 lines)
    │   ├── mod.rs                 # Core module exports (39 lines)
    │   └── tests/                 # Core tests (extracted) (399 lines)
    │       ├── mod.rs             # Test module organization (27 lines)
    │       ├── conflict_tests.rs  # Conflict detection tests (141 lines)
    │       ├── validator_tests.rs # Validation tests (152 lines)
    │       └── types_tests.rs     # Type system tests (79 lines)
    ├── ui/                        # GTK4 GUI (MVC pattern) (~3,270 lines)
    │   ├── app.rs                 # Main window coordination (208 lines)
    │   ├── actions.rs             # GTK action setup (121 lines)
    │   ├── builders/              # UI builder modules (483 lines total)
    │   │   ├── mod.rs             # Module exports (7 lines)
    │   │   ├── header.rs          # Header bar with menu (32 lines)
    │   │   ├── layout.rs          # Main layout construction (104 lines)
    │   │   └── handlers.rs        # Event handler wiring (340 lines)
    │   ├── controller.rs          # MVC Controller (mediates Model ↔ View) (590 lines)
    │   ├── file_watcher.rs        # Live config file monitoring (57 lines)
    │   ├── style.css              # GTK CSS styling (95 lines)
    │   ├── mod.rs                 # UI module exports (45 lines)
    │   ├── components/            # Reusable UI widgets (1,821 lines)
    │   │   ├── keybind_list.rs    # Scrollable list (216 lines)
    │   │   ├── search_bar.rs      # Real-time search (69 lines)
    │   │   ├── conflict_panel.rs  # Warning banner (257 lines)
    │   │   ├── details_panel.rs   # Shows selected binding (450 lines)
    │   │   ├── edit_dialog.rs     # Edit/Add dialog (411 lines)
    │   │   ├── backup_dialog.rs   # Backup management (378 lines)
    │   │   └── mod.rs             # Component exports (40 lines)
    │   └── tests/                 # UI component tests (extracted) (150 lines)
    │       ├── mod.rs             # Test module organization (21 lines)
    │       └── controller_tests.rs # Controller unit tests (129 lines)
    └── ipc/                       # Hyprland IPC integration (~590 lines)
        ├── mod.rs                 # HyprlandClient (add/remove/reload bindings) (389 lines)
        └── tests/                 # IPC tests (extracted) (201 lines)
            └── mod.rs             # IPC integration tests (201 lines)
```

For detailed architecture documentation, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

### Testing

```bash
# Run all tests (166 passing, 7 ignored)
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_conflict_detection

# Run linter
cargo clippy

# Format code
cargo fmt

# Sync version numbers across documentation
./scripts/sync-version.sh
```

### Version Management

**Fully automated release process** - one command does everything:

```bash
# Create a new release (updates all files, commits, and tags)
./scripts/tag-release.sh 1.1.0

# Then push to GitHub
git push origin main && git push origin v1.1.0
```

**What happens automatically:**
1. ✅ Updates `Cargo.toml` version (single source of truth)
2. ✅ Syncs version to all documentation files:
   - README.md (version badge)
   - SECURITY.md
   - docs/ARCHITECTURE.md
   - docs/DESIGN_DECISIONS.md
3. ✅ Updates `Cargo.lock` via `cargo check`
4. ✅ Commits all changes with proper message
5. ✅ Creates git tag `vX.Y.Z`
6. ⚠️  Reminds you to push to GitHub (manual step)

**Manual version sync** (if needed):
```bash
./scripts/sync-version.sh  # Reads version from Cargo.toml and updates docs
   ```

**Test Coverage**:
- ✅ Parser: 42 tests (handles comments, includes, multi-line, edge cases)
- ✅ Conflict Detection: 18 tests (HashMap correctness, normalisation)
- ✅ Security Validation: 50 tests (all three layers, entropy detection, edge cases)
- ✅ Config Management: 38 tests (atomic writes, backups, transactions)
- ✅ UI Components: 10 tests (controller, helper functions)
- ✅ Utilities: 7 tests (timestamp formatting, helpers)

### Code Quality Standards

- **100% Safe Rust**: No `unsafe` blocks in entire codebase
- **Safe Unwrap Usage**: One safe `unwrap()` in production (validated with preceding check), all other `unwrap()` calls in tests only
- **Clippy Clean**: Zero warnings with strict lints enabled
- **British English**: Consistent spelling in code, comments, and documentation
- **Comprehensive Documentation**: All public APIs documented with examples

---

## Technical Highlights

> **For potential employers and code reviewers**

### Design Patterns Demonstrated

1. **[MVC](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller) Architecture**: Clean separation of concerns (Model, View, Controller)
2. **[Builder Pattern](https://en.wikipedia.org/wiki/Builder_pattern)**: GTK4 widget construction with fluent API
3. **[Command Pattern](https://en.wikipedia.org/wiki/Command_pattern)**: Future undo/redo system (planned Phase 6.6)
4. **[Transaction Pattern](https://en.wikipedia.org/wiki/Database_transaction)**: [ACID](https://en.wikipedia.org/wiki/ACID) properties for config file operations
5. **[Observer Pattern](https://en.wikipedia.org/wiki/Observer_pattern)**: GTK signal handlers for reactive UI updates

### Performance Optimisations

- **[O(1)](https://en.wikipedia.org/wiki/Big_O_notation#Orders_of_common_functions) Conflict Detection**: HashMap-based instead of [O(n²)](https://en.wikipedia.org/wiki/Big_O_notation#Orders_of_common_functions) nested loops
- **[Lazy Evaluation](https://en.wikipedia.org/wiki/Lazy_evaluation)**: Conflict detection only when keybindings change
- **Efficient Parsing**: Nom combinators with [zero-copy](https://en.wikipedia.org/wiki/Zero-copy) string slicing
- **Normalised Keys**: Pre-normalised KeyCombo for consistent hashing

### Security Considerations

- **Defence in Depth**: Three independent validation layers
- **Whitelist Validation**: Deny by default, allow explicitly
- **Entropy Detection**: [Shannon entropy](https://en.wikipedia.org/wiki/Entropy_(information_theory)) catches encoded payloads (innovation)
- **Atomic Operations**: Config never corrupted, even on power loss
- **Memory Safety**: Rust's [ownership system](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) prevents entire classes of vulnerabilities

### Modern Rust Practices

- **Error Handling**: `Result<T, E>` with [`thiserror`](https://docs.rs/thiserror/) for custom errors
- **[Interior Mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)**: [`Rc<RefCell<T>>`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) for shared state in GTK callbacks
- **Lifetime Annotations**: `'a` lifetimes prevent [dangling references](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#dangling-references)
- **Trait Derivation**: `#[derive(Debug, Clone, PartialEq, Eq, Hash)]`
- **Module Organisation**: Clear separation by domain (config, core, ui)

---

## Roadmap

### Current Status: Phase 7.2 Complete ✅ (v1.0.7)

All planned features are implemented and production-ready. Documentation is now 100% verified and accurate:

- ✅ **Export/Import**: Replace and Merge modes for sharing configs
- ✅ **Conflict Resolution UI**: Advanced visual conflict resolution
- ✅ **Live Hyprland IPC**: `hyprctl reload` integration (no restart needed)
- ✅ **Live File Monitoring**: Automatic UI refresh on external config changes
- ✅ **Three-Layer Security**: Complete validation system

### Future Enhancements (Post v1.1.0)

Potential features being considered for future releases:

- Undo/redo system for keybinding changes
- Multi-monitor awareness and per-monitor bindings
- Plugin architecture for custom validators
- Keybinding suggestion system (show available key combinations)

---

## Project Policies

### API Stability & Versioning

**Semantic Versioning**: This project follows [Semantic Versioning 2.0.0](https://semver.org/):
- **MAJOR** (v2.0.0): Breaking changes to config file format or public APIs
- **MINOR** (v1.1.0): New features, backward-compatible changes
- **PATCH** (v1.0.1): Bug fixes, no new features

**Current Status (v1.0.7)**:
- Config file format is **stable** - no breaking changes planned for v1.x
- Public Rust APIs (for library use) are **unstable** - may change before v2.0.0
- CLI interface is **stable** - commands and options will remain backward-compatible

**Deprecation Policy**:
- Deprecated features will be supported for at least one MINOR version
- Warnings will be shown in CLI output before removal
- **Migration Guides**: Detailed upgrade guides will be provided for all major/minor releases in release notes and `CHANGELOG.md` (to be created)

### Accessibility

**Current Features**:
- **Keyboard Navigation**: Full application control via Tab, Arrow keys, Enter
- **GTK Accessibility**: Inherits GTK4's built-in accessibility support (screen readers, high contrast)
- **Clear Visual Feedback**: Conflict warnings, validation errors with colour coding

**Future Plans**:
- WCAG 2.1 Level AA compliance evaluation
- Keyboard shortcut customisation
- Improved screen reader support with ARIA-like labeling

### Internationalization (i18n)

**Current Status**: English only

**Future Plans** (Post v1.1.0):
- **Planned**: Support for multiple languages using Rust `fluent` or `gettext` crates
- **UI Strings**: All user-facing text will be extracted to translation files
- **Priority Languages**: Based on community demand (likely German, French, Spanish first)

**Contributions Welcome**: If you're interested in translating the application, please open an issue to discuss.

---

## Documentation

This project maintains comprehensive technical documentation for developers, contributors, and security researchers.

### Core Documentation

| Document | Description | Audience |
|----------|-------------|----------|
| **[README.md](README.md)** | Project overview, installation, usage, and quick start | Users & Contributors |
| **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** | System design, component interaction, data flow diagrams | Developers |
| **[DESIGN_DECISIONS.md](docs/DESIGN_DECISIONS.md)** | Rationale for major architectural choices and trade-offs | Developers & Architects |
| **[SECURITY.md](SECURITY.md)** | Security architecture, threat model, vulnerability reporting | Security Researchers |
| **[CONTRIBUTING.md](CONTRIBUTING.md)** | How to report bugs, request features, and give feedback | Contributors |

### Deep Dives

| Document | Description | Lines | Status |
|----------|-------------|-------|--------|
| **[ENTROPY_DETECTION.md](docs/ENTROPY_DETECTION.md)** | Shannon entropy theory, implementation, empirical validation | 944 | Complete |

**ENTROPY_DETECTION.md** provides a comprehensive technical analysis of using Shannon entropy for detecting obfuscated malicious commands (base64/hex encoding). It covers:
- Shannon entropy fundamentals and mathematical foundation
- Implementation decisions (threshold selection, detection order)
- Theory vs. practice gap (why empirical validation matters)
- Comprehensive empirical validation (166 passing tests, 7 ignored edge cases)
- Future research directions

This document demonstrates rigorous engineering methodology: theory-guided, empirically-validated security design.

### Quick Navigation

- **Want to understand the system?** → [ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **Want to know why decisions were made?** → [DESIGN_DECISIONS.md](docs/DESIGN_DECISIONS.md)
- **Want to report a security issue?** → [SECURITY.md](SECURITY.md)
- **Want to contribute or give feedback?** → [CONTRIBUTING.md](CONTRIBUTING.md)
- **Curious about entropy detection?** → [ENTROPY_DETECTION.md](docs/ENTROPY_DETECTION.md)

---

## Contributing

This project is **open to suggestions and feedback**. While direct contributions are not yet being accepted, the author welcomes:

- **Bug reports**: Please open an issue with reproduction steps
- **Feature requests**: Describe your use case and desired behaviour
- **Security reports**: See [SECURITY.md](SECURITY.md) for responsible disclosure

The project may open to contributions in the future. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

This project is licensed under the **Apache License 2.0**.

See the [LICENSE](LICENSE) file for full details.

---

## Acknowledgements

This project stands on the shoulders of giants. Special thanks to:

### Core Technologies
- **[Hyprland](https://hypr.land/)**: Vaxry and contributors for the outstanding dynamic tiling Wayland compositor
- **[GTK Project](https://www.gtk.org/)**: GNOME Foundation for the powerful and accessible GUI toolkit
- **[Rust Language](https://www.rust-lang.org/)**: The Rust Foundation and community for memory-safe systems programming
- **[Cargo](https://doc.rust-lang.org/cargo/)**: The Rust package manager that makes dependency management effortless

### Key Libraries
- **[Nom](https://github.com/rust-bakery/nom)**: Geal (Geoffroy Couprie) for the elegant parser combinator library
- **[gtk4-rs](https://gtk-rs.org/)**: The GTK Rust bindings team for comprehensive Rust GTK4 bindings
- **[Clap](https://github.com/clap-rs/clap)**: For the ergonomic command-line argument parsing
- **[Serde](https://serde.rs/)**: For the powerful serialization framework
- **[thiserror](https://github.com/dtolnay/thiserror)**: David Tolnay for elegant error handling
- **[notify](https://github.com/notify-rs/notify)**: For cross-platform filesystem event monitoring
- **[atomic-write-file](https://github.com/google/atomic-write-file)**: Google for safe atomic file operations

### Documentation & Standards
- **[OWASP](https://owasp.org/)**: For security testing methodology and best practices
- **[MITRE ATT&CK](https://attack.mitre.org/)**: For threat intelligence framework
- **[Shannon (1948)](https://en.wikipedia.org/wiki/Claude_Shannon)**: Claude Shannon for information theory foundations
- **[GNOME HIG](https://developer.gnome.org/hig/)**: For human interface design guidelines

### Community & Ecosystem
- **Rust Community**: For exceptional tooling, documentation, and support
- **Arch Linux Community**: For comprehensive packaging and documentation
- **Wayland Developers**: For the modern display server protocol
- **Open Source Contributors**: Everyone who maintains the libraries this project depends on

---

## Contact

**Developer**: Eric Jingryd
**Email**: tidynest@proton.me
**System**: Arch Linux (TidyNest)

---

## Project Stats

### Codebase
- **Primary Language**: Rust 1.83+
- **Total Project Files**: 51 Rust files + 8 Markdown files + 4 shell scripts + 5 config files
- **Total Lines of Code**: 16,222 lines across entire project
  - Rust source code: 10,564 lines (51 .rs files: 36 production + 15 test modules)
  - Documentation: 5,044 lines (8 .md files)
  - Shell scripts: 432 lines (4 .sh files)
  - Configuration: 182 lines (.toml, .css, .conf files)
- **Production Code**: 7,795 lines (Rust code excluding tests)
- **Test Code**: 2,769 lines (26% of Rust code, 15 test modules)
- **Documentation Coverage**: 100% of public APIs with examples

### Quality Metrics
- **Test Coverage**: 166 passing tests (124 unit + 42 doc tests)
  - 7 edge cases marked as ignored
  - 0 failures
- **Security**: 3-layer validation system (injection prevention, danger detection, entropy analysis)
- **Performance**: O(1) conflict detection, <5μs for 500 bindings
- **Dependencies**: 29 direct dependencies, all security-audited
- **Build Time**: ~8-9 seconds (clean release build on modern hardware)

### Architecture
- **Design Pattern**: Model-View-Controller (MVC)
- **Modules**: 5 main modules (config, core, ui, ipc, bin)
- **Components**: 7 reusable UI widgets
- **Atomic Operations**: All file writes use atomic rename strategy
- **Live Monitoring**: inotify-based file watching with zero CPU overhead when idle

---

**Made with 🦀 Rust and GTK4**

*Professional-grade configuration management for Hyprland users*
