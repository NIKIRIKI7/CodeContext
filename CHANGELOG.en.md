<div align="center">

**[🇷🇺 Русский](./CHANGELOG.md)** · **[🇬🇧 English](./CHANGELOG.en.md)** · **[📖 Documentation (README)](./README.en.md)**

</div>

# Changelog

All notable changes to **CodeContext** are documented in this file (synchronized with the Russian edition in [`CHANGELOG.md`](./CHANGELOG.md)).
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Automated Generation:** Run `just changelog <version>` (or `just tag-release <version>`) to automatically populate both `CHANGELOG.md` (RU) and `CHANGELOG.en.md` (EN) from Git commit history.

---

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
