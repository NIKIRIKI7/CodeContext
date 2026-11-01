<div align="center">

<img src="./assets/readme/hero.svg" width="100%" alt="CodeContext Hero Banner" />

<br />

**[🇷🇺 Русский](./README.md)** · **[🇬🇧 English](./README.en.md)** · **[📋 Changelog (RU)](./CHANGELOG.md)** · **[📋 Changelog (EN)](./CHANGELOG.en.md)** · **[🏛️ Архитектура и DDD](./docs/README.md)**

[![CI & Multi-Platform Build](https://github.com/NIKIRIKI7/CodeContext/actions/workflows/ci.yml/badge.svg)](https://github.com/NIKIRIKI7/CodeContext/actions/workflows/ci.yml)
[![Release Distribution](https://github.com/NIKIRIKI7/CodeContext/actions/workflows/release.yml/badge.svg)](https://github.com/NIKIRIKI7/CodeContext/releases)
[![Rust 2024](https://img.shields.io/badge/Rust-2024_Edition-e08250?logo=rust)](./Cargo.toml)
[![GUI](https://img.shields.io/badge/GUI-GPUI_120_FPS-3b82f6)](./crates/ui)
[![License](https://img.shields.io/badge/License-MIT_%2F_Apache--2.0-10b981)](./LICENSE-MIT)

</div>

---

## 01 · Что такое CodeContext и зачем он существует

Разработчики всё чаще используют веб-чаты ИИ с огромными контекстными окнами (**1M–2M+ токенов**: Google AI Studio, Claude, ChatGPT, DeepSeek, Qwen) для архитектурного рефакторинга целых репозиториев без привязки к дорогим по-токенным API.

Но ручной цикл работы с большими проектами разрушает продуктивность:
1. **Долгий сбор контекста и риск утечки секретов:** ручное копирование десятков файлов занимает минуты, а случайное попадание `.env`, `id_rsa` или `*.pem` в буфер обмена компрометирует ключи.
2. **Зависание браузера на мегабайтных промптах:** вставка 4 МБ сырого текста с комментариями и артефактами `target/` или `node_modules/` перегружает вкладку чата.
3. **Оборванные и разноформатные ответы ИИ:** веб-чаты добавляют разговорную преамбулу, обрывают код на середине файла из-за лимита выходных токенов и смешивают форматы (`Markdown`, `SEARCH / REPLACE`, `Unified Diff`).

**CodeContext** решает эти задачи детерминированно, локально и мгновенно — **без встроенных LLM, облачных аккаунтов и сетевой телеметрии**:

<p align="center">
  <img src="./assets/readme/workflow.svg" width="100%" alt="CodeContext Workflow" />
</p>

---

## 02 · В чём преимущество (Инженерные доказательства)

Каждое архитектурное обещание проекта проверяется автоматически в CI (`dhat`, `divan`, `proptest`, `architecture.rs`):

| Характеристика | Как реализовано в CodeContext | Доказательство в коде и тестах |
| :--- | :--- | :--- |
| **Сверхскорость (`~21 мс` на 4 МБ)** | Двухфазный параллельный обход `ignore` + `rayon` и SIMD-фильтрация бинарных файлов по первым 8 КБ (`memchr`). | `crates/scanner/benches/pack_throughput.rs` (SLA `< 1 сек`, факт `~21 мс` на 1 000 файлов) |
| **Ровно 1 аллокация кучи (`0 Reallocs`)** | Точный предрасчёт байтовой ёмкости (`calculate_exact_capacity`) перед сборкой буфера, включая минификацию и `CRLF → LF`. | `crates/scanner/tests/heap_allocations.rs` (`dhat::assert_eq!(delta_blocks, 1)`) |
| **Защита от утечки секретов (`SecretGuard`)** | Многоступенчатая цепочка правил блокирует `.env*`, SSH-ключи, сертификаты и заголовки `PRIVATE KEY` внутри текста; игнорирует попытки разблокировки из чужих `.ctxpack.json` (Сценарий A3). | `crates/core/src/packing/services/secret_guard.rs` |
| **6-этапный нечёткий патчер уровня Zed** | Цепочка `Exact` → `LineEnding` → `Trimmed` → `SyntaxNormalized` → `Ellipsis (...)` → `Sørensen–Dice Bigram Similarity` с защитой от частичного слияния. | `crates/apply/src/patching/fuzzy_matcher.rs` |
| **Транзакционность и Песочница (`Sandbox`)** | Жёсткая блокировка Path Traversal (`../`, `C:\`, UNC, NTFS ADS, симлинков наружу), запись через временный файл + `sync_all` + атомарный `rename` и откат **Undo** из `.ctxpack/backups/`. | `crates/apply/tests/proptest_invariants.rs` (1 500 фаззинг-сценариев `proptest`) |
| **GPU-интерфейс 120 FPS (`GPUI`)** | Виртуализированное дерево на 50 000 файлов (`FlatTreeProjection`, пересчёт чекбокса `< 0.5 мс`) и просмотрщик мультимегабайтного текста (`VirtualTextBuffer`) с 0 аллокаций на кадр. | `crates/ui/tests/ui_state_and_projections.rs` |

---

## 03 · Какие фичи уже реализованы в проекте

### Модуль A — Упаковка контекста (`packing`)
- [x] Полная поддержка иерархических `.gitignore` и `.ignore` + автоигнорирование `node_modules`, `target`, `.git`, `dist`, `build`, `.venv`.
- [x] Каскадная конфигурация 3 уровней: `Built-in Defaults` → глобальный профиль пользователя → проектный `.ctxpack.json` (с миграцией схем `v0 → v1` и защитой от файлов `> 1 МБ`).
- [x] Потоковая минификация с учётом синтаксиса языка (схлопывание однострочных комментариев для Rust/TS/Go/C++ и сохранение отступов для Python/YAML/Shell/TOML).
- [x] AVX2/SIMD-нормализация переводов строк `CRLF → LF` на лету без промежуточных аллокаций.
- [x] Сегментированная разбивка на тома (`Chunker`: 1 том / `512 КБ` / `1 МБ` / `2 МБ`) без разрыва файлов посередине.
- [x] Генерация оглавления `=== PROJECT TREE ===` и оценка объёма в байтах, строках и токенах.

### Модуль B — Применение патчей (`applying`)
- [x] Цепочка парсеров (`FORMAT_EXTRACTOR_CHAIN`): канонические блоки `=== FILE: ... ===`, блоки Markdown с путём в заголовке или первой строке комментария (`// filepath: ...`), блоки `<<<<<<< SEARCH ... >>>>>>> REPLACE`, патчи `Unified Diff` (`--- a/... +++ b/...`) и директивы создания/удаления файлов (`=== CREATE FILE: ... ===`, `=== DELETE FILE: ... ===`).
- [x] Детерминированный **Автофикс (`Autofix`)**: очистка разговорной преамбулы ИИ, распаковка внешних обёрток ```` ```markdown ````, автоматическое закрытие оборванных блоков с пометкой `TruncatedWarning`.
- [x] Предпросмотр построчного `Unified Diff` с цветовой индикацией статусов (`Ready`, `TruncatedWarning`, `SearchBlockConflict`, `BlockedBySandbox`) и проверкой незакоммиченных изменений `git status`.
- [x] Автоматическое создание снимков в `.ctxpack/backups/<timestamp>/manifest.json` и мгновенный откат (**Undo**).

### Интеграция с ОС и Жизненный цикл (`platform` & `ui`)
- [x] **Каскадное контекстное меню ОС** из 3 действий (*«Скопировать (с .gitignore)»*, *«Применить патч (автофикс)»*, *«Открыть в GUI»*) для **Windows Explorer** (реестр `HKCU\Software\Classes` с защитой `%V\.`), **macOS Finder** (`~/Library/Services/CodeContext Copy.workflow`) и **Linux** (KDE Dolphin `kio/servicemenus`, Cinnamon Nemo, GNOME Nautilus).
- [x] **Single-Instance IPC** на локальных сокетах/каналах ОС с бинарным Zero-Copy протоколом `CC\x01` и автоматическим восстановлением после падения мёртвого сокета.
- [x] **Глобальный хоткей** (`Ctrl+Shift+C` / `⌘⇧C`) с записью сочетаний клавиш из UI и детекцией ограничений Wayland.
- [x] **Автообновление и самоинсталляция (`SystemUpdater`)** по каналам `Stable` / `Beta` / `Alpha` с проверкой SHA-256 и криптографической подписи манифеста `releases-index.json`.
- [x] Переключение темы (**Тёмная / Светлая**) и языка интерфейса (**Русский / English**) на лету.

---

## 04 · Как скачать и установить

Готовые оптимизированные сборки (с PGO и Fat LTO) публикуются на странице **[GitHub Releases](https://github.com/NIKIRIKI7/CodeContext/releases/latest)** для всех популярных платформ:

| Платформа и архитектура | Доступные форматы пакетов и установщиков | Быстрая установка |
| :--- | :--- | :--- |
| **Windows x64** (`x86_64-pc-windows-msvc`) | `.msi` (инсталлятор WiX), `.zip`, `.7z`, портативный `.exe` | Запустите `.msi` (автоматически пропишет `PATH` и меню Проводника) или запустите `codecontext.exe --self-install` |
| **Windows ARM64** (`aarch64-pc-windows-msvc`) | `.zip`, `.7z`, портативный `.exe` | Распакуйте архив и запустите `codecontext.exe --self-install` |
| **macOS Apple Silicon** (`M1 / M2 / M3 / M4`) | `.dmg`, `.pkg`, `.zip`, `.tar.gz` | Перетащите `CodeContext.app` из `.dmg` в `/Applications` или установите `.pkg` |
| **macOS Intel & Universal2** (`x86_64` / `universal`) | `.dmg`, `.pkg`, `.zip`, `.tar.gz` | Установите `codecontext-universal-apple-darwin.dmg` |
| **Ubuntu / Debian / Mint / Pop!_OS** (`amd64` & `arm64`) | `.deb`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst` | `sudo dpkg -i codecontext-*-linux-gnu.deb` |
| **Fedora / RHEL / Rocky / openSUSE** (`x86_64` & `aarch64`) | `.rpm`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst` | `sudo rpm -Uvh codecontext-*-linux-gnu.rpm` |
| **Arch Linux / Manjaro / SteamOS** (`x86_64` & `aarch64`) | `.pkg.tar.zst`, `.AppImage`, `.tar.gz` | `sudo pacman -U codecontext-*-linux-gnu.pkg.tar.zst` |

> **Проверка целостности:** В каждом релизе приложен файл `SHA256SUMS.txt` и подписанный манифест `releases-index.json`.

---

## 05 · Как скомпилировать из исходников

### 1. Системные зависимости
Установите стабильный тулчейн **Rust (1.88+ / Edition 2024)** через [rustup.rs](https://rustup.rs/).

- **Windows 10 / 11:** Visual Studio Build Tools (C++ workload) и встроенный `rust-lld`.
- **macOS:** Xcode Command Line Tools (`xcode-select --install`).
- **Linux (Ubuntu / Debian):**
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    build-essential binutils lld clang libclang-dev pkg-config \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libvulkan-dev \
    libfontconfig1-dev libfreetype6-dev libx11-dev libxcursor-dev libxi-dev
  ```
- **Linux (Fedora / RHEL):**
  ```bash
  sudo dnf install -y gcc binutils lld clang clang-devel pkgconf-pkg-config \
    libxcb-devel libxkbcommon-devel libxkbcommon-x11-devel wayland-devel \
    vulkan-loader-devel fontconfig-devel freetype-devel libX11-devel libXcursor-devel libXi-devel
  ```
- **Linux (Arch / Manjaro):**
  ```bash
  sudo pacman -S --needed base-devel lld clang pkgconf libxcb libxkbcommon \
    libxkbcommon-x11 wayland vulkan-icd-loader fontconfig freetype2 libx11 libxcursor libxi
  ```

### 2. Сборка и запуск

```bash
# Клонирование репозитория
git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext

# Сборка максимально оптимизированного релизного бинарника (Fat LTO, CGU=1, Strip)
cargo build --release --locked --package app

# Запуск графического интерфейса (через cargo-алиас `r`)
cargo r

# Прогон всех тестов, архитектурного линтера DDD и проверки 1 аллокации кучи (dhat)
cargo test --package app --test architecture
cargo test --release --package scanner --test heap_allocations -- --test-threads=1
```

---

## 06 · Краткая инструкция: CLI, Headless и GUI

### А. Командная строка (CLI) и Headless Fast-Path

Бинарник поддерживает мгновенный безголовый режим (`headless`) для скриптов, терминала и контекстного меню ОС:

```bash
# 1. Мгновенно упаковать проект в системный буфер обмена (без открытия GUI)
codecontext --pack-headless /path/to/project

# 2. Применить ответ ИИ из буфера обмена к проекту (с автофиксом и бэкапом)
codecontext --apply-headless /path/to/project

# 3. Открыть указанную папку проекта в графическом окне GPUI
codecontext --project /path/to/project

# 4. Зарегистрировать / удалить каскадное меню в Проводнике / Finder / Dolphin / Nemo
codecontext --register-shell
codecontext --unregister-shell

# 5. Выполнить самоинсталляцию в системный каталог пользователя и проверить обновления
codecontext --self-install
codecontext --check-update
codecontext --version
```

### Б. Графический интерфейс (GPUI) и горячие клавиши

| Сочетание клавиш (Win/Linux · macOS) | Действие |
| :--- | :--- |
| `Ctrl+Shift+C` · `⌘⇧C` | **Глобальный хоткей** (фокус окна / копирование контекста / авточтение буфера) |
| `Ctrl+O` · `⌘O` | Открыть папку проекта |
| `Ctrl+F` · `⌘F` | Мгновенный поиск по дереву файлов (`Backspace` / `Esc` — сброс) |
| `F5` или `Ctrl+R` · `⌘R` | Повторное сканирование файловой системы проекта |
| `Ctrl+V` · `⌘V` | Вставить ответ ИИ из буфера обмена и автоматически запустить разбор + автофикс |
| `Ctrl+Enter` · `⌘↵` | Транзакционно применить распознанные изменения на диск |
| `Ctrl+Z` · `⌘Z` | Откатить последнее применение из снимка `.ctxpack/backups/` (**Undo**) |

---

## 07 · Архитектура воркспейса (DDD + Ports & Adapters)

Воркспейс разделён на **8 строго изолированных крейтов**, границы которых защищены архитектурными фитнес-функциями (`crates/app/tests/architecture.rs`) и `deny.toml`:

```text
crates/
├── core/       # Чистое доменное ядро (0 внешних инфраструктурных зависимостей):
│               # поддомены packing, applying, preset, system, shared_kernel
├── scanner/    # Адаптер параллельного обхода ФС (ignore, rayon, memchr SIMD)
├── apply/      # Адаптер парсинга ответов ИИ, 6-этапного Fuzzy-патчинга, Sandbox и бэкапов
├── config/     # Каскадный загрузчик JSON (Defaults -> Global -> .ctxpack.json) и миграции
├── platform/   # Интеграция с ОС (буфер arboard, хоткеи, Single-Instance IPC, меню оболочки)
├── di/         # Composition Root (контейнер внедрения зависимостей AppContainer)
├── ui/         # Графический интерфейс GPUI по паттерну State-Entity-View (120 FPS)
└── app/        # Точка входа бинарника, CLI-роутер и архитектурные фитнес-тесты
```

Подробная документация находится в каталоге **[`docs/`](./docs/README.md)**.

---

## Лицензия

Проект распространяется на условиях двойной лицензии **[MIT](./LICENSE-MIT) OR [Apache-2.0](./LICENSE-APACHE)**.