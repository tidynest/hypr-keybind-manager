 # Contributing to Hyprland Keybinding Manager

Thank you for your interest in contributing to Hyprland Keybinding Manager! This document outlines how you can provide feedback, report issues, and potentially contribute to the project in the future.

## Table of Contents

- [Current Contribution Status](#current-contribution-status)
- [How to Provide Feedback](#how-to-provide-feedback)
  - [1. Reporting Bugs](#1-reporting-bugs)
  - [2. Suggesting Features](#2-suggesting-features)
  - [3. Security Reports](#3-security-reports)
- [Development Setup (For Future Contributors)](#development-setup-for-future-contributors)
  - [Prerequisites](#prerequisites)
  - [Setup](#setup)
  - [Project Structure](#project-structure)
- [Code of Conduct](#code-of-conduct)
  - [Expected Behaviour](#expected-behaviour)
  - [Unacceptable Behaviour](#unacceptable-behaviour)
- [Questions?](#questions)
- [Future Contribution Guidelines (When Accepting Contributions)](#future-contribution-guidelines-when-accepting-contributions)
  - [Code Quality Standards](#code-quality-standards)
  - [Testing Requirements](#testing-requirements)
  - [Commit Message Format](#commit-message-format)
  - [Pull Request Process](#pull-request-process)
- [Recognition](#recognition)
- [License](#license)
- [Contact](#contact)
- [Acknowledgements](#acknowledgements)

---

## Current Contribution Status

**The project is currently NOT accepting direct code contributions**, but the author welcomes:

✅ **Bug Reports** - Help identify issues and edge cases
✅ **Feature Suggestions** - Propose new functionality or improvements
✅ **Security Reports** - Responsible disclosure of vulnerabilities
✅ **Documentation Feedback** - Suggest improvements to docs
✅ **General Feedback** - Share your experience using the application

**Future Plans**: The project may open to community contributions after reaching version 1.0 and establishing clear contribution guidelines.

---

## How to Provide Feedback

### 1. Reporting Bugs

Before submitting a bug report, please:
- Check existing issues to avoid duplicates
- Test with the latest version from the `main` branch
- Reproduce the bug with a minimal test case

**Create a GitHub Issue** with:

```markdown
**Description**: Brief summary of the bug

**Steps to Reproduce**:
1. Launch application with config file X
2. Click "Add Keybinding"
3. Enter invalid key combination
4. Click "Save"

**Expected Behaviour**: Error dialog shows helpful message

**Actual Behaviour**: Application crashes with panic

**Environment**:
- OS: Arch Linux 6.12.5
- Rust Version: 1.83
- GTK4 Version: 4.16
- Hyprland Version: 0.45

**Logs** (if applicable):
```
Paste crash output or error messages
```

**Config File** (if relevant):
Attach config file or minimal reproducible config
```

---

### 2. Suggesting Features

Feature suggestions are highly valued! Please:
- Describe the **use case** (what problem does it solve?)
- Explain the **expected behaviour** (how should it work?)
- Consider **alternatives** (other ways to solve the problem?)

**Create a GitHub Issue** with:

```markdown
**Feature**: One-line summary (e.g., "Undo/Redo support")

**Use Case**: I frequently make mistakes when editing keybindings and
wish I could undo changes without manually restoring from backups.

**Proposed Solution**: Add Undo/Redo buttons to the main window that
track the last 10 config changes in memory.

**Alternatives Considered**:
- Restore from backup (current workflow, but requires multiple clicks)
- Manual config file editing (error-prone)

**Additional Context**: This feature would be especially helpful when
experimenting with new keybinding layouts.
```

---

### 3. Security Reports

**DO NOT open public GitHub issues for security vulnerabilities.**

**Email security reports to**: tidynest@proton.me
**Subject Line**: `[SECURITY] Hyprland Keybinding Manager - [Brief Description]`

**Include**:
- Vulnerability description
- Steps to reproduce
- Proof-of-concept (if applicable)
- Suggested fix (if you have one)

See [SECURITY.md](SECURITY.md) for the full responsible disclosure process.

---

## Development Setup (For Future Contributors)

If you're interested in exploring the codebase or preparing for future contributions:

### Prerequisites

- **[Rust](https://www.rust-lang.org/) 1.83+** with [Cargo](https://doc.rust-lang.org/cargo/)
- **[GTK4](https://www.gtk.org/) 4.0+** development libraries
- **[Git](https://git-scm.com/)** for version control

### Setup

```bash
# Clone the repository
git clone https://github.com/tidynest/hypr-keybind-manager.git
cd hypr-keybind-manager

# Build the project
cargo build

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Project Structure

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for complete project structure and design patterns.

**Quick Reference**:
- `src/core/` - Business logic (parsing, conflict detection, validation)
- `src/config/` - File I/O and security validation
- `src/ui/` - GTK4 user interface
- `tests/` - Unit and integration tests

---

## Code of Conduct

### Expected Behaviour

- **Be Respectful**: Treat everyone with respect and kindness
- **Be Professional**: Keep discussions focused on technical matters
- **Be Constructive**: Provide helpful feedback with actionable suggestions
- **Be Patient**: The maintainer works on this project in spare time

### Unacceptable Behaviour

- **Harassment**: Personal attacks, insults, or derogatory comments
- **Spam**: Repeated off-topic issues or comments
- **Bad Faith**: Intentionally wasting maintainer time
- **Entitlement**: Demanding features or immediate responses

**Consequences**: Violation of code of conduct may result in issue closure or ban from the project.

---

## Questions?

- **General Questions**: Open a GitHub Discussion
- **Technical Questions**: Review [ARCHITECTURE.md](docs/ARCHITECTURE.md) and [DESIGN_DECISIONS.md](docs/DESIGN_DECISIONS.md)
- **Security Questions**: Email tidynest@proton.me

---

## Future Contribution Guidelines (When Accepting Contributions)

*This section is a preview of planned contribution requirements for when the project opens to contributors.*

### Code Quality Standards

All contributions must meet these standards:

✅ **Pass All Tests**: `cargo test` returns 0 failures
✅ **Pass Linter**: `cargo clippy` returns 0 warnings
✅ **Formatted**: `cargo fmt` applied
✅ **Documented**: All public APIs have doc comments
✅ **Type-Safe**: No unsafe `unwrap()` in production code
✅ **Memory-Safe**: Zero `unsafe` blocks (unless absolutely necessary with justification)

### Testing Requirements

- **Unit Tests**: All new functions must have unit tests
- **Integration Tests**: New features must have integration tests
- **Edge Cases**: Tests must cover error conditions and edge cases
- **No Regressions**: Existing tests must continue passing

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Examples**:
```
feat(ui): add undo/redo buttons to main window
fix(parser): handle escaped quotes in arguments
docs(security): update threat model with new vulnerabilities
test(conflict): add test for normalised key combo equality
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

### Pull Request Process

1. **Fork** the repository
2. **Create a branch** from `main`: `git checkout -b feat/your-feature`
3. **Make changes** with commits following message format
4. **Add tests** covering your changes
5. **Update documentation** if needed
6. **Run quality checks**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
7. **Push to your fork**: `git push origin feat/your-feature`
8. **Create Pull Request** with:
   - Clear description of changes
   - Reference to related issue (if applicable)
   - Screenshots (for UI changes)
   - Test plan

**Review Process**:
- Maintainer reviews code within 7 days
- Address review comments
- Once approved, maintainer merges

---

## Recognition

Contributors will be recognised in:
- `CONTRIBUTORS.md` file (future)
- Release notes for significant contributions
- Special thanks in README for major features

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project ([Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)).

---

## Contact

**Maintainer**: Eric Jingryd
**Email**: tidynest@proton.me
**System**: Arch Linux (TidyNest)

---

## Acknowledgements

Thank you for taking the time to read this document and for your interest in improving Hyprland Keybinding Manager!

Even though direct contributions aren't being accepted yet, your **feedback, bug reports, and feature suggestions** are incredibly valuable and help shape the project's future direction.

**Every issue, suggestion, and security report makes this project better for everyone.** ✅

---

**Last Updated**: 2025-10-19
**Version**: 1.0.4
