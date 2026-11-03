<div align="center">

**[🇷🇺 Русский](./CHANGELOG.md)** · **[🇬🇧 English](./CHANGELOG.en.md)** · **[📖 Documentation (README)](./README.en.md)**

</div>

# Changelog

All notable changes to **CodeContext** are documented in this file (synchronized with the Russian edition in [`CHANGELOG.md`](./CHANGELOG.md)).
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Automated Generation:** Run `just changelog <version>` (or `just tag-release <version>`) to automatically populate both `CHANGELOG.md` (RU) and `CHANGELOG.en.md` (EN) from Git commit history.

---

## [0.2.0] - 2026-11-03

### Added
- **First-Class Folder Filtering & Top-Down Pruning (`crates/core`, `crates/scanner`)**: Two-level filtering funnel (`IncludeSubtree`, `TraverseOnly`, `PruneSubtree`) that prunes excluded directories (`node_modules`, `target`, `.git`, globs, regexes) on entry without reading their subtrees.
- **Transparent Ancestor Transit (`TraverseOnly`)**: Specific `include` rules (e.g., `crates/core/src`) transparently traverse through excluded parent folders (`crates/`) without leaking sibling directories or parent files.
- **Configuration `Schema v2` (`crates/config`)**: Dedicated `filters` (file extensions, globs, regexes) and `folder_filters` (`exclude_names`, `exclude_globs`, `only_folders`, `include_regex_folders`, `exclude_regex_folders`, and `folder_overrides` supporting both literal paths and globs like `**/node_modules`) with seamless `v0 -> v1 -> v2` migration.
- **Minimalist On-Demand Regex & Filter Bar (`crates/ui`)**: Filter pills remain hidden by default and only expand as a single-line capped strip (`+N` overflow counter) when the `[.*]` regex mode is toggled.
- **Clean Tree Projection (`FlatTreeProjection`, `TemplateEngine`)**: Automatic empty-folder pruning, folders-first alphabetical ordering, and compact intermediate path collapsing (`crates/core/src`).

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