# Refactoring Steps - Hyprland Keybinding Manager

**Status:** In Progress
**Started:** 2025-10-23
**Goal:** Improve modularity by breaking down large monolithic files

---

## 📊 Initial Analysis

### Files Requiring Refactoring (by priority)

| File | Lines | Priority | Status |
|------|-------|----------|--------|
| `src/config/mod.rs` | 2011 → 570 | 🔴 Critical | ✅ **Complete (71.6% reduction!)** |
| `src/config/danger.rs` | 1619 | 🔴 Critical | ⏳ Not Started |
| `src/ui/app.rs` | 746 | 🟡 Medium | ⏳ Not Started |
| `src/ui/controller.rs` | 649 | 🟢 Low | ⏳ Not Started |

**Total lines to refactor:** ~5,025 lines
**Target reduction:** ~60% (down to ~2,000 lines with better organisation)

---

## 🎯 Phase 1: src/config/mod.rs (2011 → 570 lines) ✅ **COMPLETE**

**Completed:** 2025-10-23

**Original Structure:**
```
src/config/mod.rs (2011 lines)
├─ ConfigError enum (88 lines)
├─ ConfigManager (608 lines)
├─ ConfigTransaction (500 lines)
└─ Tests (1000+ lines)
```

**Achieved Structure:**
```
src/config/
├─ mod.rs (570 lines - core ConfigManager)
├─ error.rs (90 lines - ConfigError enum)
├─ transaction.rs (330 lines - ConfigTransaction)
└─ tests/
    ├─ mod.rs
    ├─ config_manager_tests.rs (530 lines)
    └─ transaction_tests.rs (470 lines)
```

**Results:**
- **Reduction:** 2011 → 570 lines (71.6% reduction, 1441 lines removed)
- **Tests:** All 124 tests passing
- **Clippy:** Clean, no warnings

### Step 1.1: Create Directory Structure
- [x] Create `src/config/tests/` directory

### Step 1.2: Extract ConfigError
- [x] Create `src/config/error.rs`
- [x] Move `ConfigError` enum definition (lines 47-87)
- [x] Update `mod.rs` to use `pub use error::ConfigError;`
- [x] Run `cargo check` to verify compilation

### Step 1.3: Extract ConfigTransaction
- [x] Create `src/config/transaction.rs`
- [x] Move `ConfigTransaction` struct and impl (lines 647-943)
- [x] Update `mod.rs` to use `pub use transaction::ConfigTransaction;`
- [x] Run `cargo check` to verify compilation

### Step 1.4: Extract Backup Methods
- [x] **Skipped** - Methods are well-integrated with ConfigManager, no benefit to extraction

### Step 1.5: Reorganise Tests
- [x] Create `src/config/tests/mod.rs`
- [x] Create `src/config/tests/config_manager_tests.rs`
- [x] Create `src/config/tests/transaction_tests.rs`
- [x] Move tests to appropriate files
- [x] Run `cargo test` to verify all tests pass

### Step 1.6: Final Verification
- [x] Run `cargo clippy` - no new warnings
- [x] Run `cargo test` - all 124 tests pass
- [x] Verify git status shows only expected changes

---

## 🎯 Phase 2: src/config/danger.rs (1619 → ~300 lines)

**Current Structure:**
```
src/config/danger.rs (1619 lines)
├─ DangerLevel + DangerAssessment (125 lines)
├─ DangerDetector (450 lines)
├─ Pattern building (150 lines)
├─ Entropy calculations (250 lines)
└─ Tests (600+ lines)
```

**Target Structure:**
```
src/config/danger/
├─ mod.rs (~150 lines - DangerDetector core + assess_command)
├─ types.rs (~50 lines - DangerLevel, DangerAssessment)
├─ patterns.rs (~150 lines - build_*_patterns/commands functions)
├─ entropy.rs (~200 lines - calculate_entropy, is_likely_*)
└─ tests/
    ├─ mod.rs
    ├─ critical_patterns_tests.rs
    ├─ entropy_tests.rs
    └─ integration_tests.rs
```

### Step 2.1: Create Directory Structure
- [ ] Rename `src/config/danger.rs` to `src/config/danger_backup.rs` (temp)
- [ ] Create `src/config/danger/` directory
- [ ] Create `src/config/danger/tests/` directory

### Step 2.2: Extract Types
- [ ] Create `src/config/danger/types.rs`
- [ ] Move `DangerLevel` enum (lines 102-112)
- [ ] Move `DangerAssessment` struct (lines 115-125)
- [ ] Create `src/config/danger/mod.rs` with `pub use types::*;`

### Step 2.3: Extract Pattern Builders
- [ ] Create `src/config/danger/patterns.rs`
- [ ] Move `build_critical_patterns()` (lines 434-449)
- [ ] Move `build_dangerous_commands()` (lines 467-485)
- [ ] Move `build_suspicious_commands()` (lines 502-530)
- [ ] Move `build_safe_commands()` (lines 549-591)
- [ ] Make these public functions
- [ ] Update `mod.rs` imports

### Step 2.4: Extract Entropy Module
- [ ] Create `src/config/danger/entropy.rs`
- [ ] Move `calculate_entropy()` (lines 680-711)
- [ ] Move `is_likely_base64()` (lines 778-820)
- [ ] Move `is_likely_hex()` (lines 917-939)
- [ ] Make these public functions
- [ ] Update `mod.rs` imports

### Step 2.5: Refactor DangerDetector
- [ ] Update `DangerDetector::new()` to call pattern builders
- [ ] Keep `assess_command()` in `mod.rs`
- [ ] Keep `check_dangerous_arguments()` in `mod.rs`
- [ ] Remove `danger_backup.rs` after verification

### Step 2.6: Reorganise Tests
- [ ] Create test modules following new structure
- [ ] Move critical pattern tests
- [ ] Move entropy tests
- [ ] Move integration tests
- [ ] Run `cargo test` to verify

### Step 2.7: Final Verification
- [ ] Run `cargo clippy`
- [ ] Run `cargo test`
- [ ] Verify all functionality intact

---

## 🎯 Phase 3: src/ui/app.rs (746 → ~300 lines)

**Current Structure:**
```
src/ui/app.rs (746 lines)
├─ App struct (43 lines)
├─ load_css() (15 lines)
└─ build_ui() (650+ lines)
    ├─ Actions setup
    ├─ Widget creation
    ├─ Event handlers
    └─ Button wiring
```

**Target Structure:**
```
src/ui/
├─ app.rs (~150 lines - App struct + main flow)
├─ actions.rs (~150 lines - GTK actions)
├─ builders/
│   ├─ mod.rs
│   ├─ header.rs (~50 lines - HeaderBar)
│   ├─ layout.rs (~100 lines - Paned structure)
│   └─ handlers.rs (~250 lines - button handlers)
```

### Step 3.1: Create Directory Structure
- [ ] Create `src/ui/builders/` directory

### Step 3.2: Extract Actions Module
- [ ] Create `src/ui/actions.rs`
- [ ] Extract quit action setup (lines 138-143)
- [ ] Extract export action setup (lines 170-202)
- [ ] Extract import action setup (lines 329-376)
- [ ] Create helper functions with clear signatures

### Step 3.3: Extract Header Builder
- [ ] Create `src/ui/builders/header.rs`
- [ ] Extract HeaderBar creation (lines 147-160)
- [ ] Make it a standalone function returning HeaderBar

### Step 3.4: Extract Layout Builder
- [ ] Create `src/ui/builders/layout.rs`
- [ ] Extract left panel creation (lines 264-288)
- [ ] Extract details panel creation (line 302)
- [ ] Extract Paned configuration (lines 304-324)

### Step 3.5: Extract Event Handlers
- [ ] Create `src/ui/builders/handlers.rs`
- [ ] Extract delete handler (lines 466-540)
- [ ] Extract edit handler (lines 554-602)
- [ ] Extract add handler (lines 616-675)
- [ ] Extract backup handler (lines 689-741)

### Step 3.6: Refactor build_ui()
- [ ] Reduce `build_ui()` to coordinate calls
- [ ] Each section becomes a function call
- [ ] Should be ~50 lines of coordination logic

### Step 3.7: Final Verification
- [ ] Run `cargo clippy`
- [ ] Run `cargo build`
- [ ] Test GUI functionality manually
- [ ] Verify all interactions work

---

## 🎯 Phase 4: src/ui/controller.rs (649 lines - Minor cleanup)

**Action:** Only extract tests to separate file

### Step 4.1: Extract Tests
- [ ] Create `src/ui/controller_tests.rs`
- [ ] Move all tests (lines 537-649)
- [ ] Update imports

### Step 4.2: Final Verification
- [ ] Run `cargo test`
- [ ] Verify all tests pass

---

## ✅ Final Checklist

- [ ] All phases complete
- [ ] Run `cargo clippy` on entire project
- [ ] Run `cargo test` - all 165 tests pass
- [ ] Run `cargo build --release` - no warnings
- [ ] Manual GUI testing - all features work
- [ ] Update CLAUDE.md with refactoring completion notes
- [ ] Commit changes with descriptive message
- [ ] Delete this file (`docs/refactoring_steps.md`)

---

## 📈 Expected Results

**Before:**
- 4 files over 600 lines
- Hardto navigate and understand
- Tests mixed with implementation
- Multiple concerns per file

**After:**
- ~15-20 smaller, focused files
- Each file has one clear responsibility
- Tests organised by module
- Easy to find and modify specific functionality
- Better code organisation following Rust best practises

**Lines of Code:**
- Before: ~5,025 lines (monolithic)
- After: ~2,000 lines (well-organised)
- Reduction: ~60% perceived complexity

---

**Notes:**
- Run `cargo check` after every major change
- Run `cargo test` after completing each phase
- Keep commits small and focused
- If anything breaks, git revert and try smaller steps
