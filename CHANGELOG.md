# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Security
Fixes from an internal audit on 2026-10-14. Upgrade if you import bind files from other people.

- High: dispatcher arguments written into `hyprland.lua` could run code when Hyprland loaded the file. The check evaluated them without globals, so `os and os.execute("...") or 1` passed as the number 1, and then the typed text was written. Arguments are now evaluated as data and the resulting values are written back by `src/core/lua_literal.lua`, never the typed text.
- High: an imported bind could add lines to `hyprland.conf`. A Lua string with a line break became a second directive, such as `exec-once`, and a comma in a key or description moved the following fields, so another dispatcher ran. The writer and the export now refuse both.
- High: none of the validation described in SECURITY.md ran when a bind was saved. Adding, editing and importing now refuse commands the danger detector rates critical, for `exec` and `execr`. An import with such a bind stops before anything is written.
- Medium: the Bubblewrap toggle only sandboxed the first command of `a; b`, because Hyprland hands exec text to a shell. The command line now goes, quoted, to a shell inside the sandbox. The sandbox also mounts `/lib` and `/lib64`, without which no dynamically linked program could start inside it.
- Medium: a `#` in an edited bind was written as is and commented out the rest of the line. It is written as `##` now.
- Medium: backups were created with default permissions, so a `0600` config got world-readable copies. They keep the config's permissions now.
- Medium: the restore list offered backups of other files in the same directory, so restoring could put `gaming.lua` over `hyprland.lua`. Only the edited file's backups are listed, and symlinks are skipped.
- Medium: in a Lua export, a bind that runs Lua code is written as a comment, and a line break in its description ended that comment early, leaving the rest as code. Line breaks are flattened.
- Medium: the Lua argument check had no instruction limit, so an endless loop froze the GUI. It has the config parser's limit.
- Low: the danger detector let a known safe first word hide a critical command (`kitty -e rm -rf /`), and never looked at `execr`.
- Low: `list` and `check` printed control characters from the config as they were, so listing someone else's config could send escape sequences to the terminal.
- Low: Lua keywords as table keys and non-finite numbers in arguments came back as invalid Lua.
- Low: the workspace lints in `Cargo.toml` were never applied, because the package had no `[lints] workspace = true`. They are now, which removed an `unsafe` block in `main.rs` and two `expect` calls that could abort on an unusual config path.
- Low: the release workflow used actions by mutable tag and gave the whole workflow a write token. Actions are pinned to commit SHAs, credentials are not kept after checkout, the build uses `--locked`, and Dependabot proposes action updates.

### Fixed
- `cargo test` could not fail in this repository. `.cargo/runner.sh`, which filters GTK warnings, runs test binaries too, and it returned the status of the last command in its pipeline, `awk`. A failing test run therefore exited 0, in the pre-push gate and in the PKGBUILD's `check()` as well. The runner now exits with the program's own status. The suite was green when this was found.

## [1.4.1] - 2026-09-06

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
