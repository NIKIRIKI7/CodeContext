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
- **Interactive Filter Modal & Live Regex Validation (`crates/ui`, `crates/scanner`, `crates/di`)**: Extension filtering modes (`All files`, `Include only`, `Exclude selected`), manual extension and folder addition, one-click regex presets (`No tests`, `Only src/`, `No generated`, `Only crates/`), and real-time `is_valid_regex_pattern` syntax validation.
- **Composable System Prompts & Custom Keymap (`crates/core`, `crates/ui`)**: Multi-block prompt builder (`PromptBlockItem`) with category filtering (`Patcher`, `Architecture`, `AI Behavior`, `Custom`) and reordering, plus customizable per-action keyboard shortcuts (`KeymapConfig`).
- **Expanded Filter/Regex Benchmark Suite & `Justfile` Targets (`crates/scanner`, `crates/ui`, `Justfile`)**: Added `bench_6c`–`bench_6e` and `bench_7` benchmarks covering regex matrices, extension whitelists/blacklists, and 50,000-file live tree search, alongside granular test and benchmark recipes in `Justfile`.

### Performance
- **Directory Checkbox Toggle on 50,000 Files (`Project::set_directory_selected`)**: Accelerated by **23.6×** (`883 µs → 37.39 µs`, `1.34 Gitem/s`) using `partition_point` binary search instead of a full linear scan.
- **Fuzzy Patcher `fuzzy_indent` (`crates/apply`)**: Accelerated by **5.8×** (`22.54 ms → 3.88 ms`) while reducing heap allocations by **947×** (`120,267 → 127 allocs`) via a zero-string-allocation `TrimmedLine` index and lazy `Option::None` normalization.
- **Filter Funnel & Live Regex Validation (`CompiledFilterPlan`)**: 10,000-path evaluation in `bench_6a` accelerated by **4.45×** (`4.16 ms → 894 µs`, `11.18 Mitem/s`, `0 allocs`) by converting `StackNormalizedPath` to an `enum` (eliminating 1 KB stack zeroing) and fast-matching `**/<folder>` segments; live regex validation accelerated by **1,103×** (`551 µs → 499.6 ns`, `0 allocs`) via a zero-allocation delimiter balance pre-check and an 8-slot `thread_local` cache.
- **Tree Projection & 4 MB In-Memory Repack (`FlatTreeProjection`, `PackService`)**: 4 MB live preview repack accelerated by **1.82×** (`14.25 ms → 7.82 ms`, `539.5 MB/s` peak); heap memory during 50k-file directory toggling reduced by **6.3×** (`11.4 MB → 1.80 MB`) and during 50k-file live search by **18.9×** (`5.55 MB → 299.9 KB`, `2.61 ms`).
- **Sandbox Path Validation (`validate_sandbox_path`)**: Accelerated by **1.89×** (`5.27 ms → 2.79 ms` per 10,000 paths) with **26% lower memory usage** by rejecting `..` traversal segments prior to `PathBuf` allocation.

### Fixed
- **`SystemUpdater` Isolation & Binary Auto-Recovery (`crates/platform`, `crates/app`)**: Isolated `SystemUpdater::new_in_memory()` from the host's `canonical_install_binary_path()`, enforced native executable magic-header verification (`MZ` / `ELF` / `Mach-O`), and added automatic self-healing from `.old` or `target/release` if a corrupted binary is detected.
- **Subfolder Exclusion Inheritance inside `Include` Overrides (`crates/scanner`)**: `exclude_names` (`node_modules`), `exclude_globs` (`**/fixtures`), and `folder_exclude_regex` now properly prune descendant folders inside an explicitly included parent `folder_overrides` branch.

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
- **Multi-Platform CI/CD & Auto-Updater**: PGO + Fat LTO release builds across Windows (`x64`, `ARM64` — `.msi`, `.zip`, `.7z`, `.exe`), macOS (`Apple Silicon`, `Intel`, `Universal2` — `.dmg`, `.pkg`, `.zip`, `.tar.gz`), Linux (`x86_64`, `ARM64` — `.deb`, `.rpm`, `.pkg.tar.zst`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst`), plus FreeBSD and multi-distro verification.