# CodeContext AI

![CodeContext AI](assets/images/logo.png)

[![AUR](https://img.shields.io/aur/version/codecontext-ai)](https://aur.archlinux.org/packages/codecontext-ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue)](https://www.python.org/downloads/)

## About

**CodeContext AI** is a powerful desktop tool for preparing your codebase to work with Large Language Models (LLMs) like ChatGPT, Claude, Cursor, and others. It scans project folders, analyzes file structure, builds dependency graphs, and generates a single, perfectly structured prompt — optimized for token consumption and architectural clarity.

### Why was this product created?

When working with neural networks, developers constantly face context window token limits and the problem of AI "losing" architectural coherence when code is copied in parts. CodeContext AI solves this by letting you collect your entire project into one perfectly structured prompt in just a few clicks.

### What problems does it solve?

- **Token optimization** — Minify, remove comments/secrets, and use Skeleton mode to strip implementations while preserving structure (up to 80% token reduction)
- **Architectural clarity** — Include file tree, dependency graph (AST/Regex), and Mermaid architecture diagrams
- **Safe LLM patching** — Apply AI-suggested code changes directly to your files with safety diff preview and built-in LLM verification
- **Context management** — Remember recent workspaces, custom presets, and prompt templates

### Key differentiators

| Feature | CodeContext AI | Manual / Alternatives |
|---|---|---|
| Smart compression (Minify + Skeleton) | Up to 80% token reduction | Manual copy-paste |
| LLM Patcher | JSON-based patch preview & apply | Not available |
| LLM Checker | Auto-verify code before saving | Not available |
| Dependency graph (AST) | Python, JS/TS, Vue understanding | File listing only |
| OS integration | Context menu (Windows/Linux) | None |
| Theme system | Apple, Modern, custom JSON themes | Fixed UI |

## Features

- **GUI & CLI modes** — Full PySide6 desktop UI with drag-and-drop, or terminal/CI/CD usage
- **Smart file scanning** — Respects `.gitignore`, filters by extension, detects changed files via Git
- **Token estimation** — Uses `tiktoken` (OpenAI algorithms) for accurate context cost prediction
- **Customizable output** — Markdown, XML, Plain Text, JSONL chunks, or custom Jinja2 templates (11 built-in)
- **LLM Patch system** — Paste AI JSON response, preview changes in interactive diff viewer, apply safely
- **Architecture analysis** — AST-based import graph, Mermaid diagram generation, file tree
- **Theme engine** — Built-in themes (Apple, Modern), dark/light mode, import custom JSON themes
- **UI customization (v1.14+)** — Show/hide sidebar tabs and action buttons (Premiere Pro-style interface settings)
- **Cross-platform** — Windows, Linux, macOS
- **AUR package** — Install directly from Arch User Repository

## Installation

### From AUR (Arch Linux)

```bash
yay -S codecontext-ai
# or
paru -S codecontext-ai
```

### From source

#### Prerequisites
- Python 3.10 or higher
- Git (for repository cloning)

#### Steps

```bash
git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext

python -m venv venv
# Windows:
venv\Scripts\activate
# Linux/macOS:
source venv/bin/activate

pip install -r requirements.txt
```

## Usage

### GUI mode

```bash
python main.py
```

1. Drag & drop a project folder into the window (or use "+ Папка ПК")
2. Files are automatically scanned — use the file tree to exclude individual files
3. Configure filters: extensions, ignored paths, Minify, No Comments, No Secrets, Skeleton mode
4. Choose output format: Markdown (default), XML, Plain, or custom Jinja2 template
5. Click **"📋 В Буфер обмена"** and paste into ChatGPT / Claude
6. Alternatively, click **"💻 В редактор"** to open in VS Code / Cursor

### CLI mode

```bash
# Basic scan
python main.py --cli --path /path/to/your/project

# With dependency mode (default, shallow, deep)
python main.py --cli --path /path/to/project --mode deep

# Apply LLM JSON patch
python main.py --cli --path /path/to/project --patch /path/to/llm_response.json

# Using the installed CLI tool (Linux)
codecontext --cli --path /path/to/project
```

### UI customization (v1.14+)

Click the **⚙** (gear) button next to the version label in the sidebar to open the Interface Settings dialog. From there you can toggle visibility of individual sidebar tabs and action buttons — just like customizing workspace in Premiere Pro or VS Code.

## Tech Stack

- **Language:** Python 3.10+
- **GUI Framework:** PySide6 (Qt 6)
- **Architecture:** Clean Architecture + Redux-like unidirectional data flow (Store → Controller → UI)
- **Key libraries:** `asyncio` (async I/O), `tiktoken` (token counting), `jinja2` (templating), `PySide6` (Qt bindings)

## Project Structure

```
CodeContext/
├── main.py                  # Application entry point
├── VERSION.txt              # Current version
├── requirements.txt         # Python dependencies
├── assets/                  # Icons, images
│   └── images/logo.png
├── aur_build/               # AUR packaging
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # Built-in theme JSON files
└── src/
    ├── main_app.py          # App bootstrap & startup
    ├── store/               # Redux-like state management
    │   ├── state.py         # Data models (AppSettings, AppState)
    │   └── store.py         # Store with subscribe/dispatch
    ├── controllers/         # Business logic layer
    │   └── main_controller.py
    ├── ui/                  # PySide6 UI layer
    │   ├── main_window.py   # Main window with splitter layout
    │   ├── dialogs.py       # All modal dialogs
    │   ├── theme_manager.py # Theme engine
    │   └── components/      # Reusable widgets
    │       ├── sidebar.py       # Settings sidebar (dynamic tabs)
    │       ├── action_panel.py  # Action buttons (dynamic visibility)
    │       ├── folder_list.py
    │       ├── file_tree.py
    │       ├── log_panel.py
    │       ├── status_bar.py
    │       ├── empty_state.py
    │       └── analytics_panel.py
    └── utils/
        ├── config.py        # Config, presets, version info
        ├── async_runtime.py # Async event loop bridge
        └── ...
```

## Roadmap

- [ ] macOS Finder context menu integration (Automator)
- [ ] Direct OpenAI/Anthropic API integration (send prompt without clipboard)
- [ ] Hexagonal Architecture analysis strategy
- [ ] Plugin system for custom analyzers and exporters
- [ ] i18n / multi-language support

## Contributing

Contributions are welcome! Please follow the existing code style (Clean Architecture with Redux-like state management).

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Please ensure your code follows SOLID principles and the architectural patterns documented in `docs/ARCHITECTURE.md`.

## Team & Contact

**Developer:** mcniki
**Contact:** [VK: gor_niki](https://vk.com/gor_niki) | Issues & PRs on GitHub

## License

Distributed under the MIT License. See `LICENSE` for more information.
