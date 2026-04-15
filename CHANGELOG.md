# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

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
