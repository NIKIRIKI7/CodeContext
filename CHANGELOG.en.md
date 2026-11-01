<div align="center">

**[🇷🇺 Русский](./CHANGELOG.md)** · **[🇬🇧 English](./CHANGELOG.en.md)** · **[📖 Documentation (README)](./README.en.md)**

</div>

# Changelog

All notable changes to **CodeContext** are documented in this file (synchronized with the Russian edition in [`CHANGELOG.md`](./CHANGELOG.md)).
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Automated Generation:** Run `just changelog <version>` (or `just tag-release <version>`) to automatically populate both `CHANGELOG.md` (RU) and `CHANGELOG.en.md` (EN) from Git commit history.

---

## [0.1.0] - 2026-11-01

### Added
- **Module A (`packing`)**: Two-phase parallel directory traversal (`ignore` + `rayon`) respecting hierarchical `.gitignore`, `SecretGuard` protection (Scenario A3), 8 KB SIMD binary detection (`memchr`), language-aware streaming minification, multi-volume splitting (`Chunker`), and zero-reallocation buffer assembly (`dhat`).
- **Module B (`applying`)**: Multi-format parser chain (`Canonical`, `Markdown`, `Search/Replace`, `Unified Diff`, `Create/Delete`), deterministic `Autofix`, Zed-grade 6-stage fuzzy matcher (Sørensen–Dice bigram similarity + `...` ellipsis support), `Sandbox` path validation, and transactional writes with instant `Undo` (`.ctxpack/backups`).
- **OS Integration (`platform`)**: Cascading `CodeContext` context menu with 3 actions (`Copy Context (with .gitignore)`, `Apply AI Patch (Autofix)`, `Open in GUI`) for Windows Explorer (`HKCU`), macOS Finder (`Services`), and Linux (`Dolphin`, `Nemo`, `Nautilus`), global hotkeys, and Single-Instance IPC.
- **Desktop GUI (`ui`)**: Virtualized 50,000-file tree and multi-megabyte virtual text viewer built on `gpui` (120 FPS) with runtime language (`RU / EN`) and theme (`Dark / Light`) switching.
- **Multi-Platform CI/CD & Auto-Updater**: PGO + Fat LTO release builds across Windows (`x64`, `ARM64` — `.msi`, `.zip`, `.7z`, `.exe`), macOS (`Apple Silicon`, `Intel`, `Universal2` — `.dmg`, `.pkg`, `.zip`, `.tar.gz`), Linux (`x86_64`, `ARM64` — `.deb`, `.rpm`, `.pkg.tar.zst`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst`), plus FreeBSD and multi-distro verification.