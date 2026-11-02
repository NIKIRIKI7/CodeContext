<div align="center">

**[🇷🇺 Русский](./CHANGELOG.md)** · **[🇬🇧 English](./CHANGELOG.en.md)** · **[📖 Documentation (README)](./README.en.md)**

</div>

# Changelog

All notable changes to **CodeContext** are documented in this file (synchronized with the Russian edition in [`CHANGELOG.md`](./CHANGELOG.md)).
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Automated Generation:** Run `just changelog <version>` (or `just tag-release <version>`) to automatically populate both `CHANGELOG.md` (RU) and `CHANGELOG.en.md` (EN) from Git commit history.

---

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
- **Multi-Platform CI/CD & Auto-Updater**: PGO + Fat LTO release builds across Windows (`x64`, `ARM64` — `.msi`, `.zip`, `.7z`, `.exe`), macOS (`Apple Silicon`, `Intel`, `Universal2` — `.dmg`, `.pkg`, `.zip`, `.tar.gz`), Linux (`x86_64`, `ARM64` — `.deb`, `.rpm`, `.pkg.tar.zst`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst`), plus FreeBSD and multi-distro verification.