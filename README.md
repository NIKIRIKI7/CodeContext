<div align="center">

# CodeContext AI

<img src="assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-powered codebase analysis & prompt preparation tool**

<br>

<input type="radio" name="lang" id="lang_ru" checked hidden>
<input type="radio" name="lang" id="lang_en" hidden>

<label for="lang_ru" class="lang-label lang-ru-label">🇷🇺 Русский</label>
<label for="lang_en" class="lang-label lang-en-label">🇬🇧 English</label>

<br>

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.14.0-blue?style=flat-square)](VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<!-- ==================== RUSSIAN ==================== -->

<div class="lang-ru">

<div align="center">
  <sub><i>Переключите язык вверху → 🇬🇧 English</i></sub>
</div>

<br>

## 🌟 О проекте

**CodeContext AI** — мощный десктопный инструмент для подготовки кодовой базы к работе с большими языковыми моделями (LLM), такими как ChatGPT, Claude, Cursor и другие. Он сканирует папки проекта, анализирует структуру файлов, строит графы зависимостей и генерирует единый идеально структурированный промпт — оптимизированный по токенам и архитектурной связности.

### ❓ Зачем это нужно?

При работе с нейросетями разработчики постоянно упираются в лимит контекстного окна (Token Limit), а ИИ «теряет» архитектурную связность проекта, когда код копируется частями. **CodeContext AI решает эту проблему** — в пару кликов вы собираете ВЕСЬ проект в один структурированный промпт.

### 🎯 Ключевые возможности

| Возможность | CodeContext AI | Вручную / Альтернативы |
|---|---|---|
| 🗜️ Сжатие (Minify + Skeleton) | **До 80%** экономии токенов | Копировать вручную |
| 🧩 LLM Patcher | Предпросмотр и применение JSON-патчей | Нет аналогов |
| ✅ LLM Checker | Авто-проверка кода перед сохранением | Нет аналогов |
| 🔗 Граф зависимостей (AST) | Python, JS/TS, Vue | Только список файлов |
| 🖱️ Интеграция с ОС | Контекстное меню Windows/Linux | Нет |
| 🎨 Тема оформления | Apple, Modern, кастомные JSON-темы | Фиксированный UI |
| ⚙️ Кастомизация UI (v1.14+) | Premiere Pro-style — скрывай вкладки и кнопки | Фиксированный UI |

---

## 🚀 Фичи

- **🖥️ GUI + CLI** — Полноценный интерфейс на PySide6 с Drag & Drop или работа из терминала/CI/CD
- **📂 Умное сканирование** — Учитывает `.gitignore`, фильтрует по расширениям, видит изменения через Git
- **🔢 Токенизация** — Использует `tiktoken` (алгоритмы OpenAI) для точного подсчёта стоимости контекста
- **📝 Кастомизируемый вывод** — Markdown, XML, Plain Text, JSONL-чанки, свои Jinja2-шаблоны (11 встроенных)
- **🧪 LLM Patch** — Вставь JSON-ответ нейросети, просмотри изменения в Diff-вьювере, примени безопасно
- **🏗️ Анализ архитектуры** — AST-граф импортов, Mermaid-диаграммы, дерево файлов
- **🎨 Темизация** — Встроенные темы (Apple, Modern), тёмный/светлый режим, импорт своих JSON-тем
- **⚙️ Кастомизация интерфейса (v1.14+)** — Premiere Pro-style: скрывай и показывай вкладки и кнопки действий через настройки
- **💻 Кроссплатформенность** — Windows, Linux, macOS
- **📦 AUR-пакет** — Установка одной командой на Arch Linux

---

## 📥 Установка

### Из AUR (Arch Linux)

```bash
yay -S codecontext-ai
# или
paru -S codecontext-ai
```

### Из исходников

**Требования:** Python 3.10+, Git

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

### Собрать в .exe (Windows)

```bash
pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py
```

---

## 💻 Использование

### GUI-режим

```bash
python main.py
```

1. **Перетащите** папку проекта в окно (или нажмите «+ Папка ПК»)
2. Файлы просканируются автоматически — исключайте лишние через дерево файлов
3. Настройте фильтры: расширения, игнорируемые пути, Minify, No Comments, No Secrets, Skeleton
4. Выберите формат вывода: Markdown (по умолчанию), XML, Plain или кастомный Jinja2-шаблон
5. Нажмите **«📋 В Буфер обмена»** и вставьте в ChatGPT / Claude
6. Или нажмите **«💻 В редактор»** — результат откроется в VS Code / Cursor

### CLI-режим

```bash
# Базовое сканирование
python main.py --cli --path /путь/к/проекту

# С режимом анализа зависимостей (default, shallow, deep)
python main.py --cli --path /путь/к/проекту --mode deep

# Применить JSON-патч от LLM
python main.py --cli --path /путь/к/проекту --patch /путь/к/ответу_llm.json

# Через глобальный CLI (Linux)
codecontext --cli --path /путь/к/проекту
```

### 🎛️ Кастомизация интерфейса (v1.14+)

Нажмите кнопку **⚙** (шестерёнка) рядом с версией в боковой панели — откроется диалог настройки интерфейса. Включайте и выключайте видимость вкладок и кнопок действий, как в Premiere Pro или VS Code.

---

## 🏗️ Стек технологий

| Компонент | Технология |
|---|---|
| **Язык** | Python 3.10+ |
| **GUI** | PySide6 (Qt 6) |
| **Архитектура** | Clean Architecture + Redux-like (Store → Controller → UI) |
| **Токенизация** | `tiktoken` (OpenAI) |
| **Шаблоны** | `jinja2` |
| **Асинхронность** | `asyncio` |

---

## 📁 Структура проекта

```
CodeContext/
├── main.py                  # Точка входа
├── VERSION.txt              # Версия
├── requirements.txt         # Зависимости
├── assets/
│   ├── images/logo.png
│   └── ...
├── aur_build/               # AUR-пакет
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # JSON-темы
└── src/
    ├── main_app.py          # Загрузчик приложения
    ├── store/               # Управление состоянием (Redux-style)
    │   ├── state.py
    │   └── store.py
    ├── controllers/
    │   └── main_controller.py
    ├── ui/                  # Слой PySide6
    │   ├── main_window.py
    │   ├── dialogs.py
    │   ├── theme_manager.py
    │   └── components/
    │       ├── sidebar.py
    │       ├── action_panel.py
    │       ├── folder_list.py
    │       ├── file_tree.py
    │       ├── log_panel.py
    │       ├── status_bar.py
    │       ├── empty_state.py
    │       └── analytics_panel.py
    └── utils/
        ├── config.py
        ├── async_runtime.py
        └── ...
```

---

## 🗺️ Roadmap

- [ ] 🍎 Интеграция с контекстным меню macOS (Finder/Automator)
- [ ] 🤖 Прямая интеграция с API OpenAI/Anthropic (без буфера обмена)
- [ ] 🏛️ Анализ Hexagonal Architecture
- [ ] 🔌 Плагинная система для анализаторов и экспортёров
- [ ] 🌐 i18n / интернационализация

---

## 👨‍💻 Команда

**Разработчик:** mcniki
**Связь:** [VK: gor_niki](https://vk.com/gor_niki) | Issues & PRs на GitHub

---

## 🤝 Как помочь проекту?

Мы рады любым Pull Requests!

1. Форкните репозиторий
2. Создайте ветку: `git checkout -b feature/AmazingFeature`
3. Закоммитьте: `git commit -m 'Add some AmazingFeature'`
4. Запушьте: `git push origin feature/AmazingFeature`
5. Откройте Pull Request

Пожалуйста, соблюдайте SOLID-принципы и архитектурные паттерны проекта (см. `docs/ARCHITECTURE.md`).

---

## 📄 Лицензия

Проект распространяется под лицензией MIT. Подробности — в файле `LICENSE`.

</div>

<!-- ==================== ENGLISH ==================== -->

<div class="lang-en">

<div align="center">
  <sub><i>Switch language above → 🇷🇺 Русский</i></sub>
</div>

<br>

## 🌟 About

**CodeContext AI** is a powerful desktop tool for preparing your codebase to work with Large Language Models (LLMs) like ChatGPT, Claude, Cursor, and others. It scans project folders, analyzes file structure, builds dependency graphs, and generates a single, perfectly structured prompt — optimized for token consumption and architectural clarity.

### ❓ Why was this created?

When working with AI, developers constantly face context window token limits and the problem of LLMs "losing" architectural coherence when code is copied in parts. **CodeContext AI solves this** by letting you collect your entire project into one perfectly structured prompt in just a few clicks.

### 🎯 Key differentiators

| Feature | CodeContext AI | Manual / Alternatives |
|---|---|---|
| 🗜️ Smart compression (Minify + Skeleton) | **Up to 80%** token reduction | Manual copy-paste |
| 🧩 LLM Patcher | JSON-based patch preview & apply | Not available |
| ✅ LLM Checker | Auto-verify code before saving | Not available |
| 🔗 Dependency graph (AST) | Python, JS/TS, Vue understanding | File listing only |
| 🖱️ OS integration | Context menu (Windows/Linux) | None |
| 🎨 Theme system | Apple, Modern, custom JSON themes | Fixed UI |
| ⚙️ UI customization (v1.14+) | Premiere Pro-style — toggle tabs & buttons | Fixed UI |

---

## 🚀 Features

- **🖥️ GUI + CLI** — Full PySide6 desktop UI with drag-and-drop, or terminal/CI-CD usage
- **📂 Smart file scanning** — Respects `.gitignore`, filters by extension, detects changed files via Git
- **🔢 Token estimation** — Uses `tiktoken` (OpenAI algorithms) for accurate context cost prediction
- **📝 Customizable output** — Markdown, XML, Plain Text, JSONL chunks, or custom Jinja2 templates (11 built-in)
- **🧪 LLM Patch system** — Paste AI JSON response, preview changes in interactive diff viewer, apply safely
- **🏗️ Architecture analysis** — AST-based import graph, Mermaid diagram generation, file tree
- **🎨 Theme engine** — Built-in themes (Apple, Modern), dark/light mode, import custom JSON themes
- **⚙️ UI customization (v1.14+)** — Premiere Pro-style: show/hide sidebar tabs and action buttons via settings
- **💻 Cross-platform** — Windows, Linux, macOS
- **📦 AUR package** — Single-command install on Arch Linux

---

## 📥 Installation

### From AUR (Arch Linux)

```bash
yay -S codecontext-ai
# or
paru -S codecontext-ai
```

### From source

**Prerequisites:** Python 3.10+, Git

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

### Build .exe (Windows)

```bash
pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py
```

---

## 💻 Usage

### GUI mode

```bash
python main.py
```

1. **Drag & drop** a project folder into the window (or click "+ Папка ПК")
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

### 🎛️ UI customization (v1.14+)

Click the **⚙** (gear) button next to the version label in the sidebar to open the Interface Settings dialog. From there you can toggle visibility of individual sidebar tabs and action buttons — just like customizing workspace in Premiere Pro or VS Code.

---

## 🏗️ Tech Stack

| Component | Technology |
|---|---|
| **Language** | Python 3.10+ |
| **GUI Framework** | PySide6 (Qt 6) |
| **Architecture** | Clean Architecture + Redux-like (Store → Controller → UI) |
| **Tokenization** | `tiktoken` (OpenAI) |
| **Templating** | `jinja2` |
| **Async** | `asyncio` |

---

## 📁 Project Structure

```
CodeContext/
├── main.py                  # Entry point
├── VERSION.txt              # Version
├── requirements.txt         # Dependencies
├── assets/
│   ├── images/logo.png
│   └── ...
├── aur_build/               # AUR packaging
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # JSON theme files
└── src/
    ├── main_app.py          # App bootstrap
    ├── store/               # Redux-like state management
    │   ├── state.py
    │   └── store.py
    ├── controllers/
    │   └── main_controller.py
    ├── ui/                  # PySide6 layer
    │   ├── main_window.py
    │   ├── dialogs.py
    │   ├── theme_manager.py
    │   └── components/
    │       ├── sidebar.py
    │       ├── action_panel.py
    │       ├── folder_list.py
    │       ├── file_tree.py
    │       ├── log_panel.py
    │       ├── status_bar.py
    │       ├── empty_state.py
    │       └── analytics_panel.py
    └── utils/
        ├── config.py
        ├── async_runtime.py
        └── ...
```

---

## 🗺️ Roadmap

- [ ] 🍎 macOS Finder context menu integration (Automator)
- [ ] 🤖 Direct OpenAI/Anthropic API integration (send prompt without clipboard)
- [ ] 🏛️ Hexagonal Architecture analysis strategy
- [ ] 🔌 Plugin system for custom analyzers and exporters
- [ ] 🌐 i18n / multi-language support

---

## 👨‍💻 Team

**Developer:** mcniki
**Contact:** [VK: gor_niki](https://vk.com/gor_niki) | Issues & PRs on GitHub

---

## 🤝 Contributing

Contributions are welcome!

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/AmazingFeature`
3. Commit your changes: `git commit -m 'Add some AmazingFeature'`
4. Push to the branch: `git push origin feature/AmazingFeature`
5. Open a Pull Request

Please ensure your code follows SOLID principles and project architectural patterns (see `docs/ARCHITECTURE.md`).

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

</div>

<style>
  .lang-label {
    display: inline-block;
    padding: 8px 24px;
    margin: 0 4px;
    cursor: pointer;
    border-radius: 20px;
    font-size: 15px;
    font-weight: 600;
    transition: all 0.2s ease;
    border: 2px solid #e0e0e0;
    background: #f5f5f5;
    color: #666;
  }
  .lang-label:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  .lang-ru, .lang-en {
    display: none;
  }
  #lang_ru:checked ~ .lang-ru-label {
    background: #0969da;
    border-color: #0969da;
    color: #fff;
  }
  #lang_en:checked ~ .lang-en-label {
    background: #0969da;
    border-color: #0969da;
    color: #fff;
  }
  #lang_ru:checked ~ .lang-ru {
    display: block;
  }
  #lang_en:checked ~ .lang-en {
    display: block;
  }
</style>
