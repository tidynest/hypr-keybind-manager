# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Added
- Lua binds produced by code (loops, functions, larger statements) can now be edited and deleted. The change is appended to the end of the main file as `hl.unbind("<keys>")` plus the replacement `hl.bind(...)`, so the original code stays untouched and Hyprland applies the override.

### Changed
- Hyprland IPC talks to the compositor's control socket directly with the standard library; the GPL-licensed `hyprland` crate and its vendored copy are gone, and "Apply to Hyprland" no longer needs `hyprctl` on the PATH.
- `deny.toml` allows no copyleft licences.

## [1.4.0] - 2026-09-06

### Added
- Lua configs (`hyprland.lua`, Hyprland 0.55+) are supported. The file runs in a sandboxed embedded Lua with a recording `hl` stub, so binds built with `mainMod .. " + Q"`, loops and `require`d modules are all found. Single-line `hl.bind(...)` statements can be edited and deleted in place, keeping a `mainMod ..` prefix; new binds are appended. Loop-generated binds, function actions and `local x = hl.bind(...)` are shown read-only with the reason.
- The default config path is `hyprland.conf` when it exists, otherwise `hyprland.lua`.
- Every Hyprland bind flag combination is parsed (`bindd`, `bindnt`, `bindle`, ...) instead of only six variants; a config using any other flag no longer fails to load.
- `bindd` descriptions are read, shown in the list tooltip and details panel, and editable in the dialog.
- `source =` lines are followed, relative to the including file; bindings from sourced files are shown and edited in the file they came from.
- Submaps are recorded per binding, shown as a `[name]` prefix in the list, and conflicts are detected per submap.
- Record button in the edit dialog fills the key combination from a key press.
- Searchable dispatcher list in the edit dialog, limited to what the validator accepts.
- Status banner for apply results, imports and on-disk reloads.
- Conflicting rows are highlighted; list footer shows shown/total/conflict counts.
- Keyboard shortcuts: Enter or double-click edits, Delete deletes, Ctrl+N adds, Ctrl+F focuses search.
- `list --json` prints bindings as JSON.
- A missing config file opens a file chooser instead of failing at startup.

### Changed
- Writing bindings edits the config in place: only changed lines are replaced, removed or added, so `$mainMod`, comments, indentation and ordering are kept.
- Key names keep their case (`Return`, `XF86AudioMute`) and compare case-insensitively; modifiers sort SUPER first.
- Add and Backups moved to the header bar with symbolic icons; emoji labels removed.
- Import asks for the file before the replace/merge choice; restoring a backup can be undone.
- `hyprctl reload` failures are reported in a dialog instead of only on stderr.

## [1.3.1] - 2026-04-15

### Changed
- Updated to Rust edition 2024.
- Bumped all dependencies to latest compatible versions.

## [1.3.0] - 2026-03-27

### Added
- Undo/redo system with full snapshot history (Ctrl+Z / Ctrl+Shift+Z), 20-level depth.
- Optional Bubblewrap sandboxing for `exec` bindings with no network access.
- Inline keycombo availability feedback with suggested free alternatives while editing.
- Config file permission warnings (world-readable, world-writable, wrong owner).
- ARM64 release packaging in GitHub Actions.
- Release history and contributor recognition documents.

### Changed
- Release workflow now publishes both `x86_64` and `aarch64` Linux archives.
- Release automation now stages `CHANGELOG.md` when preparing a tagged release.
- Security documentation now reflects Bubblewrap sandboxing as an implemented feature.

## [1.2.1] - 2025-11-01

### Changed
- Opened project to community contributions.
- Updated README and CONTRIBUTING documentation.
- Fixed British spelling throughout documentation.

## [1.2.0] - 2025-11-01

### Added
- Search persistence across operations.
- Escape key support for dialogs.
- GitHub Actions release workflow for automated binary builds.

### Changed
- Documentation audit and release preparation polish.

## [1.1.0] - 2025-10-28

### Added
- Export/import in merge and replace modes.
- Conflict resolution dialog.
- Live Hyprland IPC reload integration.
- Live file monitoring with automatic UI refresh on external config changes.

## [1.0.1] - 2025-10-19

### Added
- Initial stable release with CRUD operations, conflict detection, backups, and validation layers.
