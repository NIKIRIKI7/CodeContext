<div align="center">

**[🇷🇺 Русский](./CHANGELOG.md)** · **[🇬🇧 English](./CHANGELOG.en.md)** · **[📖 Documentation (README)](./README.en.md)**

</div>

# Changelog

All notable changes to **CodeContext** are documented in this file (synchronized with the Russian edition in [`CHANGELOG.md`](./CHANGELOG.md)).
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Automated Generation:** Run `just changelog <version>` (or `just tag-release <version>`) to automatically populate both `CHANGELOG.md` (RU) and `CHANGELOG.en.md` (EN) from Git commit history.

---

## [0.3.0] - 2026-11-07

### Added
- **VS Code-Style In-Row File Tree Rename & Transactional Delete (`F2` / `Delete`, `crates/ui`)**: file and directory renaming now renders directly inside the tree row (editing only the local node name without top toolbars), while `Delete` automatically snapshots files to `.ctxpack/backups` for instant `Ctrl+Z` undo.
- **Direct In-Place Line Editing in `VirtualTextView` (`LivePreview` & `DiffView`, `crates/ui`)**: lines are edited directly inside the virtualized code row with `↑` / `↓` arrow navigation across adjacent lines and zero top editor bars.
- **Edition Profiles (`Full` / `Lite` / `Custom`), Granular Module/Chrome Toggles & Zed-Style `{ } settings.json` Live Editor (`crates/core`, `crates/config`, `crates/ui`)**: added `EditionProfile`, `FeatureFlags`, and `WorkspaceChromeConfig` to toggle any module or UI bar, plus a `{ } settings.json` mode in the Settings header for live JSON editing with schema validation.
- **Modular Setup & Uninstaller (`--setup` / `--uninstall`) with System Auto-Detection (`SystemProbe`, `crates/core`, `crates/platform`, `crates/app`)**: auto-detects OS, distro, CPU architecture, `AVX2+BMI2` / `NEON` SIMD level, display server, and file manager, writing `install-manifest.json` and supporting clean uninstallation.
- **Extension Host & Plugin Subsystem (`PluginHostPort` + `SystemPluginHost` + `docs/05-plugins/`)**: declarative `plugin.json` manifests, lazy `ActivationEvent` hooks, zero-trust `PluginPermission` guards, `PluginViewSpec` 120 FPS declarative UI protocol, JSON-RPC 2.0 runner, and built-in `@git` context and Rust signature skeletonizer plugins.

## [0.2.5] - 2026-11-06

### Added
- **Zed-Style Command Palette (`Ctrl+P` / `Ctrl+Shift+P`, `crates/ui`)**: unified minimalist overlay for instant file search (`Space` to toggle selection, `Enter` to jump to preview) and command execution (`> copy`, `> apply`, `> undo`, `> terminal`, `> theme`, `> clean`, `> rename`).
- **File & Directory Renaming (`=== RENAME FILE: old -> new ===`, `crates/core`, `crates/apply`, `crates/ui`)**: AI rename/move directive parsing with dual-path `Sandbox` validation, full `Undo` rollback in `BackupManifest`, and UI file renaming.
- **Single-File Apply & Inline Edit Mode in `DiffView` (`crates/ui`, `crates/di`)**: added `Apply File` (commits only the active file tab) and `Edit` / `Force Apply` actions directly in the diff header.
- **Actionable `SearchBlockConflict` Diagnostics (`crates/apply`, `crates/ui`)**: unmatched `SEARCH` blocks now render a diagnostic diff showing the exact unmatched block alongside the closest fuzzy candidate in the target file, with `Force Apply` and `Copy Block` actions.
- **Visual `Undo` Snapshot History (`.ctxpack/backups`, `crates/apply`, `crates/ui`)**: lists recent backup snapshots with file counts and targeted rollback in the backup drawer.
- **Zero-Trust AI Command Runner & Error Prompt Templates (`crates/apply`, `crates/ui`)**: parses `=== RUN: ... ===` directives with mandatory user confirmation (`Run` / `Edit` / `Dismiss`), optional post-apply verification, and a `Copy Error for AI` button with customizable prompt templates on failed terminal runs.
- **Staged Housekeeping & Purge (`--clean`, `crates/apply`, `crates/app`, `crates/ui`)**: added the `Maintenance` settings category with live byte/item calculation for `.ctxpack/backups`, orphaned `.tmp` files, and IPC sockets, plus `codecontext --clean [all|backups|tmp]`.

## [0.2.1] - 2026-11-06

### Added
- **Structured Prompt Composer (`crates/ui`)**: added explicit fields for prompt category (`Patcher`, `Architecture`, `AI Behavior`, `Custom`), prompt title (`Title`), and prompt body (`Content`) with clipboard paste (`Ctrl+V`) support in the System Prompt Modal.
- **Interactive Mouse Drag Resizing for Panels (`crates/ui`)**: added smooth drag resizing for horizontal split panes (`SplitPane`) and vertical drag resizing for the bottom terminal/logs dock (removing the fixed height toggle button).
- **Categorized Settings Modal in Zed / VS Code Style (`crates/ui`)**: organized settings into 5 sidebar categories (`Appearance`, `Panels & Bar`, `Keymap`, `Safety`, `System & Updates`) to keep the interface minimalist and reduce cognitive load.

### Fixed
- **Terminal & Search Input Focus Retention on `Space` Key (`crates/ui`)**: replaced root `.on_click` with mouse-down handling and stopped key propagation so pressing `Space` in the command line or search input never drops input focus.
- **Bottom Bar & Modal Visual Cleanup (`crates/ui`)**: removed stray bottom border lines and empty list frames in the prompt modal and bottom status bar.

## [0.2.0] - 2026-11-04

### Added

- **First-Class Folder Filtering & Top-Down Pruning (`crates/core`, `crates/scanner`)**: Two-level filtering funnel (`IncludeSubtree`, `TraverseOnly`, `PruneSubtree`) that prunes excluded directories (`node_modules`, `target`, `.git`, globs, regexes) on entry without reading their subtrees.
- **Transparent Ancestor Transit (`TraverseOnly`)**: Specific `include` rules (e.g., `crates/core/src`) transparently traverse through excluded parent folders (`crates/`) without leaking sibling directories or parent files.
- **Configuration `Schema v2` (`crates/config`)**: Dedicated `filters` (file extensions, globs, regexes) and `folder_filters` (`exclude_names`, `exclude_globs`, `only_folders`, `include_regex_folders`, `exclude_regex_folders`, and `folder_overrides` supporting both literal paths and globs like `**/node_modules`) with seamless `v0 -> v1 -> v2` migration.
- **Interactive Filter Modal, Custom Regex Templates & Scalable `assets/` JSON Templates (`crates/ui`, `crates/config`, `crates/scanner`, `crates/di`)**: Extension filtering modes (`All files`, `Include only`, `Exclude selected`), manual addition of extensions, folders, and custom reusable regex templates (`+ Template` with optional name and target scope), real-time `is_valid_regex_pattern` validation, and scalable JSON configuration templates in `assets/*.json`.
- **Composable System Prompts & Custom Keymap (`crates/core`, `crates/ui`)**: Multi-block prompt builder (`PromptBlockItem`) with category filtering (`Patcher`, `Architecture`, `AI Behavior`, `Custom`) and reordering, plus customizable per-action keyboard shortcuts (`KeymapConfig`).
- **Expanded Filter/Regex Benchmark Suite & `Justfile` Targets (`crates/scanner`, `crates/ui`, `Justfile`)**: Added `bench_6c`–`bench_6e` and `bench_7` benchmarks covering regex matrices, extension whitelists/blacklists, and 50,000-file live tree search, alongside granular test and benchmark recipes in `Justfile`.

### Performance

- **Directory Checkbox Toggle on 50,000 Files (`Project::set_directory_selected`)**: Accelerated by **23.6×** (`883 µs → 37.39 µs`, `1.34 Gitem/s`) using `partition_point` binary search instead of a full linear scan.
- **Fuzzy Patcher `fuzzy_indent` (`crates/apply`)**: Accelerated by **5.8×** (`22.54 ms → 3.88 ms`) while reducing heap allocations by **947×** (`120,267 → 127 allocs`) via a zero-string-allocation `TrimmedLine` index and lazy `Option::None` normalization.
- **Filter Funnel & Live Regex Validation (`CompiledFilterPlan`)**: 10,000-path evaluation in `bench_6a` accelerated by **4.45×** (`4.16 ms → 894 µs`, `11.18 Mitem/s`, `0 allocs`) by converting `StackNormalizedPath` to an `enum` (eliminating 1 KB stack zeroing) and fast-matching `**/<folder>` segments; live regex validation accelerated by **1,103×** (`551 µs → 499.6 ns`, `0 allocs`) via a zero-allocation delimiter balance pre-check and an 8-slot `thread_local` cache.
- **Tree Projection & 4 MB In-Memory Repack (`FlatTreeProjection`, `PackService`)**: 4 MB live preview repack accelerated by **1.82×** (`14.25 ms → 7.82 ms`, `539.5 MB/s` peak); heap memory during 50k-file directory toggling reduced by **6.3×** (`11.4 MB → 1.80 MB`) and during 50k-file live search by **18.9×** (`5.55 MB → 299.9 KB`, `2.61 ms`).
- **Sandbox Path Validation (`validate_sandbox_path`)**: Accelerated by **1.89×** (`5.27 ms → 2.79 ms` per 10,000 paths) with **26% lower memory usage** by rejecting `..` traversal segments prior to `PathBuf` allocation.
- - **Unintended Selection Clearing on Space Key (`crates/ui`)**: fixed accidental deselection of files and directories when pressing `Space`; the key no longer toggles or clears context checkboxes during navigation or command input.

### Fixed

- **`SystemUpdater` Isolation & Binary Auto-Recovery (`crates/platform`, `crates/app`)**: Isolated `SystemUpdater::new_in_memory()` from the host's `canonical_install_binary_path()`, enforced native executable magic-header verification (`MZ` / `ELF` / `Mach-O`), and added automatic self-healing from `.old` or `target/release` if a corrupted binary is detected.
- **Subfolder Exclusion Inheritance inside `Include` Overrides (`crates/scanner`)**: `exclude_names` (`node_modules`), `exclude_globs` (`**/fixtures`), and `folder_exclude_regex` now properly prune descendant folders inside an explicitly included parent `folder_overrides` branch.

## [0.1.2] - 2026-11-02

### Added

- **Keyboard Shortcuts Modal by Action (`crates/core`, `crates/config`, `crates/ui`)**: customizable `KeymapConfig` for 8 application actions with interactive key recording (including automatic `ЙЦУКЕН -> QWERTY` layout mapping), reset to defaults, and `Ctrl+K` invocation that does not conflict with `Ctrl+Shift+C` copying.
- **Token Budget Heatmap & Top-20 Heavy Files Mode (`crates/ui`)**: graduated file-size color coding in the tree (`< 15 KB` muted, `15–50 KB` warning yellow, `> 50 KB` orange-red) and a `Top 20 ↓` flat view inside the `Filters` inspector to quickly exclude context-bloating files.
- **Keyboard Tree Navigation & Instant Preview Jump (`crates/ui`)**: navigate the file tree with `↑`/`↓`, toggle checkboxes with `Space`, fold/unfold directories with `←`/`→`, and jump directly to `=== FILE: ... ===` in the Live Preview via `Enter` or clicking a file name (zero heap allocations via $O(\log N)$ binary search over `line_offsets`).
- **Projects Menu & Cascading Recent Projects Flyout (`crates/core`, `crates/config`, `crates/ui`)**: universal top-bar `Projects ▾` menu with folder browsing (`Ctrl+O`), new project initialization, and a hover-activated cascading submenu of recent repositories (MRU up to 8 paths with `\\?\` deduplication).
- **Partial & Single-File Patch Application (`crates/di`, `crates/ui`)**: minimalist per-tab checkboxes for selective batch transactions (a conflict in one file no longer blocks applying the remaining valid files) and a single-file `Apply` button in `DiffViewComponent`.
- **Formatter-Agnostic Matcher (`crates/apply`)**: `try_formatter_agnostic_replace` stage resilient to line reflows and trailing commas introduced by `cargo fmt` and `prettier`.

### Fixed

- **Master Checkbox & Minimalist Tree Toolbar (`crates/ui`)**: the master checkbox and file counter now count only selectable text files (ignoring binary files and locked secrets), fixing the stuck `[-]` indeterminate state; secondary sorting and folder folding controls were moved out of the search bar into the `Filters` inspector.
- **Complete JSON Localization (`crates/ui`)**: moved all remaining hardcoded UI strings, tags, and error messages into `crates/ui/assets/locales.json` (`ru` / `en`).
- **Instant Non-Blocking `Apply` (`crates/apply`, `crates/ui`)**: eliminated redundant re-parsing and fuzzy matching when clicking `Apply`, prevented `.git/index.lock` contention (`--no-optional-locks` + `CREATE_NO_WINDOW`), removed excessive `FlushFileBuffers` (`sync_all`) barriers, and removed render-frame event polling contention.
- **End-to-End `SystemUpdater` (`crates/platform`, `crates/ui`)**: implemented real release asset downloading and extraction from GitHub Releases with `SHA256SUMS.txt` verification, `releases-index.json` fallback, background startup checks, and one-click binary restart.
- **Responsive Cards & Search Focus (`crates/ui`, `.github/workflows/ci.yml`)**: fixed empty-state card clipping in narrow panes (`flex_row` + `min_w(0)`), fixed search input unfocusing when clicking outside, and preserved `$PATH` across `$GITHUB_PATH` updates on `opensuse/tumbleweed` in CI.

## [0.1.1] - 2026-11-02

### Fixed

- **CLI & RAM Guard (`crates/app`, `crates/scanner`)**: Invoking `codecontext` from a terminal without arguments no longer auto-scans `current_dir()`, preventing freezes and multi-gigabyte RAM spikes when launched from `C:\` or home directories.
- **Streaming 8 KB Header Probe (`crates/scanner`)**: Binary detector inspects the first 8 KB via a stack buffer prior to allocating memory for the file body; expanded binary extensions list (office docs, presentations, datasets); oversized files are unselected by default.
- **WarningBanner Clamping (`crates/ui`)**: Fixed long file names overflowing past the right border of the warning card (smart path compaction + strict Flexbox min-width clamping); close button is permanently anchored inside the box.
- **GPUI Focus Trap & Modal Backdrops (`crates/ui`)**: File tree search input is only active when explicitly focused (`Ctrl+F`); clicking outside modal dialogs on the backdrop closes them without event leakage; added pane min-widths to `SplitPane`.
- **CI openSUSE Fix (`.github/workflows/ci.yml`)**: Added `bash`, `tar`, `gzip` to the `zypper` step, resolving exit code 127 on minimal container images.
- **Branding (`crates/ui`, `assets`)**: Added official vector icon `UiIcon::AppLogo` to the top of the `ActivityRail`.

## [0.1.0] - 2026-11-01

### Added

- **Module A (`packing`)**: Two-phase parallel directory traversal (`ignore` + `rayon`) respecting hierarchical `.gitignore`, `SecretGuard` protection (Scenario A3), 8 KB SIMD binary detection (`memchr`), language-aware streaming minification, multi-volume splitting (`Chunker`), and zero-reallocation buffer assembly (`dhat`).
- **Module B (`applying`)**: Multi-format parser chain (`Canonical`, `Markdown`, `Search/Replace`, `Unified Diff`, `Create/Delete`), deterministic `Autofix`, Zed-grade 6-stage fuzzy matcher (Sørensen–Dice bigram similarity + `...` ellipsis support), `Sandbox` path validation, and transactional writes with instant `Undo` (`.ctxpack/backups`).
- **OS Integration (`platform`)**: Cascading `CodeContext` context menu with 3 actions (`Copy Context (with .gitignore)`, `Apply AI Patch (Autofix)`, `Open in GUI`) for Windows Explorer (`HKCU`), macOS Finder (`Services`), and Linux (`Dolphin`, `Nemo`, `Nautilus`), global hotkeys, and Single-Instance IPC.
- **Desktop GUI (`ui`)**: Virtualized 50,000-file tree and multi-megabyte virtual text viewer built on `gpui` (120 FPS) with runtime language (`RU / EN`) and theme (`Dark / Light`) switching.
- **Multi-Platform CI/CD & Auto-Updater**: PGO + Fat LTO release builds across Windows (`x64`, `ARM64` — `.msi`, `.zip`, `.7z`, `.exe`), macOS (`Apple Silicon`, `Intel`, `Universal2` — `.dmg`, `.pkg`, `.zip`, `.tar.gz`), Linux (`x86_64`, `ARM64` — `.deb`, `.rpm`, `.pkg.tar.zst`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst`, `.tar.zst`), plus FreeBSD and multi-distro verification.
