<input type="radio" name="lang" id="lang_ru" checked hidden>
<input type="radio" name="lang" id="lang_en" hidden>

<div align="center">

<label for="lang_ru" class="lang-label">🇷🇺 Русский</label>
<label for="lang_en" class="lang-label">🇬🇧 English</label>

<br><br>

# CodeContext AI

<img src="assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-powered codebase analysis & prompt preparation tool**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.14.0-blue?style=flat-square)](VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<!-- ==================== RUSSIAN ==================== -->

<div class="lang-ru">

## 🌟 О проекте

**CodeContext AI** — мощный десктопный инструмент для подготовки кодовой базы к работе с большими языковыми моделями (LLM). Сканирует папки проекта, анализирует структуру, строит граф зависимостей и генерирует единый структурированный промпт.

### ❓ Зачем?

При работе с ИИ разработчики упираются в лимит токенов — нейросеть «теряет» архитектуру проекта, когда код копируется частями. **CodeContext AI решает это**: в пару кликов собирает ВЕСЬ проект в один промпт, экономя до 80% токенов.

---

## 🚀 Возможности

| Возможность | CodeContext AI | Вручную |
|---|---|---|
| 🗜️ Minify + Skeleton | **До 80%** экономии токенов | Копировать вручную |
| 🧩 LLM Patcher | Предпросмотр и JSON-патчи | Нет |
| ✅ LLM Checker | Авто-проверка кода перед записью | Нет |
| 🔗 AST-граф | Python, JS/TS, Vue | Только файлы |
| 🖱️ Контекстное меню | Windows / Linux | Нет |
| 🎨 Темы | Apple, Modern, кастомные | Фиксированный UI |
| ⚙️ Кастомизация (v1.14+) | Premiere Pro-style | Фиксированный UI |

---

## 📥 Установка

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

### Arch Linux (AUR)

```bash
yay -S codecontext-ai
```

### Windows .exe

```bash
pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py
```

---

## 💻 Работа в GUI-режиме

```bash
python main.py
```

### 1. Обзор интерфейса

Окно разделено на три зоны:
- **Левая панель (вкладки)** — настройки сканирования, фильтры, промпты, LLM, темы
- **Центральная область** — список папок, дерево файлов, аналитика токенов
- **Верхняя панель (Action Panel)** — опции Minify/No Comments/Skeleton и кнопки действий

### 2. Добавление проекта

| Действие | Как сделать |
|---|---|
| Перетащить папку | Просто перетащите папку проекта в окно |
| Выбрать через диалог | Кнопка «+ Папка ПК» на вкладке **Источники** |
| GitHub репозиторий | Кнопка «+ GitHub / PR» — вставьте URL репозитория или Pull Request |
| Сохранить конфиг | Кнопка «💾 Сохранить конфиг» — создаст `.codecontextrc` для повторного использования |

**Режимы загрузки GitHub:**
- **Сохранить навсегда** — клонирует репозиторий в выбранную папку на диске
- **Временная загрузка** — клонирует во временную папку (удаляется при закрытии программы)

### 3. Настройка сканирования

#### Вкладка «📡 Источники»

| Опция | Описание |
|---|---|
| ☑ Только Git Changes | Включить в результат только файлы, изменённые в последнем коммите |
| ☑ Учитывать .gitignore | Автоматически исключать файлы из `.gitignore` |
| 🔍 Сканировать файлы | Запустить сканирование — построить дерево файлов с метаданными |

#### Вкладка «🎯 Фильтры»

| Опция | Описание |
|---|---|
| **Пресеты расширений** | Быстрое переключение между наборами расширений (Python, Web, Golang, Rust, C# и др.) |
| **Расширения** | Кастомный список расширений файлов для включения (через пробел или с новой строки) |
| **Игнорировать пути** | Список папок/файлов для исключения (node_modules, .git, build, dist и т.д.) |
| ☑ Включить дерево файлов | Добавляет структуру папок в начало промпта |
| ☑ Включить карту зависимостей | AST-анализ импортов для Python/JS/TS — показывает связи между файлами |
| ☑ Включить Mermaid-граф | Генерирует архитектурную диаграмму в формате Mermaid |

**Сохранение кастомных пресетов:** настройте фильтры и нажмите 💾 — введите имя. Пресет появится в выпадающем списке.

#### Вкладка «📝 Промпты»

| Опция | Описание |
|---|---|
| **Пресеты промптов** | Быстрая смена системного промпта (Code Review, Bug Hunter, Refactoring, Security Audit и др.) |
| **Системный промпт** | Текстовое поле для кастомного промпта. Именно этот текст будет отправлен LLM как system-контекст |
| **🧩 Применить JSON-патч** | Вставьте JSON-ответ от LLM с изменениями — программа покажет Diff и даст применить файлы на диск |

**Использование JSON-патча:**
1. Попросите LLM вернуть JSON-массив с изменениями по формату:
   ```json
   [
     {"action": "replace", "file": "main.py", "search": "def old():", "content": "def new():"}
   ]
   ```
2. Вставьте JSON в диалог и нажмите **«Далее»**
3. Откроется **Safety Diff Viewer** — для каждого файла показаны изменения ДО/ПОСЛЕ
4. Отметьте нужные файлы галочками (или снимите, если изменение некорректно)
5. При желании нажмите **«🤖 Проверить через LLM»** — нейросеть проверит патч на ошибки
6. Если LLM предлагает улучшенную версию — появится панель с Diff между вашим патчем и вариантом ИИ
7. Нажмите **«💾 Сохранить выбранные на диск»**

### 4. Настройка формата вывода

Верхняя панель (Action Panel):

| Опция | Описание |
|---|---|
| ☑ Minify | Удаляет лишние пробелы и пустые строки |
| ☑ No Comments | Вырезает все комментарии из кода |
| ☑ No Secrets | Маскирует потенциальные секреты (ключи API, пароли, токены) |
| ☑ Skeleton ☠️ | **Удаляет тела функций**, оставляя только названия и структуру классов — максимальная экономия токенов |
| Формат | Markdown (по умолчанию), XML, Plain, JSONL Chunks, Custom (Jinja2) |
| 📁 (шаблон) | Выбор Jinja2-шаблона (активно при формате Custom) |

**Skeleton Mode** подробно:
- Удаляет реализацию функций, сохраняя только `def func_name(...):`
- Оставляет все классы с их методами и полями (но без тел методов)
- Позволяет LLM «понять» архитектуру огромного проекта, потратив минимум токенов
- Пример: `def calculate_total(price, tax):` → `# ... implementation ...`

### 5. Действия с результатом

| Кнопка | Действие |
|---|---|
| 👀 Предпросмотр | Открывает **Advanced Preview Dialog** с двумя вкладками: «Итоговый промпт» и «До/После» |
| 📋 В Буфер обмена | Копирует результат в буфер обмена — вставьте в ChatGPT / Claude |
| 🚀 Отправить в ChatGPT / Claude | Автоматически открывает веб-версию чата и вставляет контекст |
| 💻 В редактор | Открывает результат в VS Code / Cursor (настраивается в LLM & ОС) |
| 💾 В Файл | Сохраняет результат в файл на диске |

### 6. Advanced Preview Dialog

**Вкладка «📝 Итоговый промпт»:**
- **Слева:** список файлов, включённых в промпт — кликните, чтобы перейти к файлу
- **Справа:** полный текст промпта с подсветкой синтаксиса (Markdown/XML)
- **Кнопки:** «📋 Копировать всё» / «✂️ Скопировать только этот файл»

**Вкладка «🔍 До/После (Оптимизация)»:**
- Выберите файл из выпадающего списка
- Показывает цветной Diff между оригиналом и оптимизированной версией
- Счётчик экономии токенов: `Было: 1500 → Стало: 300 (Сжато на 80.0%)`

### 7. LLM & ОС (вкладка настроек)

#### LLM Checker

| Опция | Описание |
|---|---|
| ☑ Включить проверку | Включает автоматическую LLM-верификацию патчей перед их применением |
| URL | Адрес API (по умолчанию `https://api.openai.com/v1`) |
| Ключ | API-ключ (сохраняется в настройках) |
| Модель | Имя модели (`gpt-4o-mini`, `claude-3-haiku`, `llama3` и т.д.) |

**Быстрые пресеты:**
- 🦙 **Ollama** — `http://localhost:11434/v1`, модель `llama3`
- 🖥 **LM Studio** — `http://localhost:1234/v1`, модель `local-model`

#### Интеграция с ОС

| Кнопка | Описание |
|---|---|
| В меню проводника | Добавляет пункт «Open with CodeContext AI» в контекстное меню проводника |
| Удалить из меню | Удаляет пункт из контекстного меню |
| Добавить в PATH | Устанавливает глобальную команду `codecontext` для вызова из терминала |
| Удалить из PATH | Удаляет глобальную команду |
| **Редактор** | Укажите редактор: `code`, `cursor`, `idea`, `vim` (пусто — системный) |

#### Обновления

| Опция | Описание |
|---|---|
| ☑ Получать Pre-release | Подписаться на pre-release версии |
| 🔄 Обновления | Проверить наличие новой версии и скачать/установить автоматически |

### 8. Темы (вкладка настроек)

| Опция | Описание |
|---|---|
| **Тема** | Выбор темы оформления (Apple, Modern) |
| **Режим** | Светлая / тёмная тема |
| 📂 Открыть папку тем | Открывает папку, куда можно добавить свои JSON-темы |
| ➕ Импортировать тему | Загрузить `.json`-тему из файла |

Импортированные темы появляются в выпадающем списке автоматически.

### 9. Аналитика токенов

Вкладка **«📊 Аналитика токенов»** показывает таблицу со сканированными файлами:
- Путь к файлу
- Размер в токенах (через `tiktoken`, алгоритмы OpenAI)
- Размер после сжатия
- Процент экономии
- Стоимость в $ для выбранной LLM-модели (цены с сервера обновляются автоматически)

### 10. 🎛️ Кастомизация интерфейса (v1.14+)

Нажмите **⚙** рядом с версией в левой панели — откроется диалог «Настройка интерфейса (Premiere Pro style)».

**Можно включить/выключить вкладки:**
- 📡 Источники · 🎯 Фильтры · 📝 Промпты · LLM & ОС · 🎨 Темы

**Можно включить/выключить кнопки действий:**
- 👀 Предпросмотр · 📋 Буфер · 🚀 ChatGPT · 💻 Редактор · 💾 Файл

Изменения сохраняются и применяются сразу.

### 11. Командная палитра

`Ctrl+Shift+P` — быстрый доступ ко всем действиям без мыши:

| Команда | Описание |
|---|---|
| Сгенерировать: Копировать в буфер | Запустить обработку и скопировать в буфер обмена |
| Сгенерировать: Предпросмотр | Открыть Advanced Preview Dialog |
| Опции: Minify / Skeleton / Mermaid | Включить/выключить соответствующие опции |
| Действие: Применить JSON-патч | Открыть диалог JSON-патча |
| Настройки: Переключить тему | Переключить светлую/тёмную тему |
| Система: Проверить обновления | Проверить наличие новой версии |

---

## 💻 Работа в CLI-режиме

```bash
python main.py --cli --path /путь/к/проекту [опции]
```

### Справка по всем параметрам

```bash
python main.py --help
```

### Основные параметры

| Параметр | Тип | Описание | Пример |
|---|---|---|---|
| `--cli` | флаг | Включить CLI-режим (без GUI) | `--cli` |
| `--path` | сп. | Путь к проекту (можно несколько) | `--path ./app ./lib` |
| `--ext` | стр. | Расширения через пробел | `--ext ".py .js .ts"` |
| `--ignore` | стр. | Игнорируемые пути через `,` | `--ignore "node_modules,.git"` |
| `--mode` | выб. | Режим зависимостей: `none`, `default`, `shallow`, `deep` | `--mode deep` |
| `--format` | выб. | Формат: `markdown`, `xml`, `plain`, `jsonl_chunk` | `--format xml` |
| `--minify` | флаг | Включить Minify | `--minify` |
| `--no-comments` | флаг | Удалить комментарии | `--no-comments` |
| `--no-secrets` | флаг | Маскировать секреты | `--no-secrets` |
| `--skeleton` | флаг | Режим Skeleton | `--skeleton` |
| `--output` | стр. | Путь для сохранения результата | `--output result.txt` |
| `--stdout` | флаг | Вывести результат в консоль | `--stdout` |
| `--git` | флаг | Только Git-изменённые файлы | `--git` |
| `--gitignore` | флаг | Учитывать .gitignore | `--gitignore` |
| `--tree` | флаг | Включить дерево файлов | `--tree` |
| `--mermaid` | флаг | Включить Mermaid-граф | `--mermaid` |
| `--dependencies` | флаг | Включить карту зависимостей | `--dependencies` |
| `--patch` | стр. | Применить JSON-патч от LLM | `--patch patch.json` |
| `--template` | стр. | Путь к Jinja2-шаблону | `--template my.j2` |
| `--system-prompt` | стр. | Кастомный системный промпт | `--system-prompt "Рецензия"` |
| `--no-banner` | флаг | Не выводить баннер при старте | `--no-banner` |

### Примеры использования

```bash
# Минимальный запуск — результат в stdout
python main.py --cli --path ./myapp --stdout

# Полный анализ с XML-выводом в файл
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Сравнение через Git — только изменённые файлы
python main.py --cli --path ./myapp --git --gitignore --stdout

# Применить JSON-патч от ChatGPT/Claude
python main.py --cli --path ./myapp --patch llm_response.json

# Кастомный Jinja2-шаблон
python main.py --cli --path ./myapp --template custom_template.j2 --stdout

# Глубокий анализ с Mermaid-диаграммой
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Быстрое копирование в буфер (Linux)
python main.py --cli --path ./myapp --minify --no-comments --stdout | xclip -selection clipboard

# Через глобальный CLI (Linux)
codecontext --cli --path ./myapp --stdout

# Множественные пути
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml
```

---

## 🏗️ Стек технологий

| Компонент | Технология |
|---|---|
| Язык | Python 3.10+ |
| GUI | PySide6 (Qt 6) |
| Архитектура | Clean Architecture + Redux-like (Store → Controller → UI) |
| Токенизация | `tiktoken` (OpenAI) |
| Шаблоны | `jinja2` (11 встроенных шаблонов) |
| Асинхронность | `asyncio` |
| AST-парсеры | `ast` (Python), `tree-sitter` (JS/TS/Go/Rust) |
| Дистрибуция | PyInstaller (Windows .exe), AUR (Arch Linux) |

---

## 📁 Структура проекта

```
CodeContext/
├── main.py                  # Точка входа приложения
├── VERSION.txt              # Версия (читается при запуске)
├── requirements.txt         # Python-зависимости
├── assets/
│   └── images/logo.png      # Логотип
├── aur_build/               # AUR-пакет для Arch Linux
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # JSON-файлы встроенных тем
└── src/
    ├── main_app.py          # Загрузчик: инициализация Store, Controller, окна
    ├── store/               # Redux-like управление состоянием
    │   ├── state.py         # Data classes: AppSettings, AppState, ProcessedFile
    │   └── store.py         # Store с подпиской (subscribe/dispatch)
    ├── controllers/         # Бизнес-логика и оркестрация
    │   └── main_controller.py
    ├── ui/                  # Слой PySide6 (все визуальные компоненты)
    │   ├── main_window.py   # Главное окно с QSplitter
    │   ├── dialogs.py       # Все диалоговые окна
    │   ├── theme_manager.py # Загрузка/применение тем
    │   └── components/
    │       ├── sidebar.py       # Левая панель (5 вкладок с настройками)
    │       ├── action_panel.py  # Верхняя панель (чекбоксы + кнопки)
    │       ├── folder_list.py   # Список добавленных папок
    │       ├── file_tree.py     # Дерево файлов с чекбоксами исключений
    │       ├── log_panel.py     # Панель логов (слева-внизу)
    │       ├── status_bar.py    # Строка состояния (токены, стоимость, прогресс)
    │       ├── empty_state.py   # Пустое состояние с Drag & Drop
    │       └── analytics_panel.py # Таблица токенов и стоимости
    └── utils/
        ├── config.py        # Пресеты, версия, пути, PricingManager
        ├── async_runtime.py # Мост asyncio → Qt event loop
        └── ...
```

---

## 🗺️ Roadmap

- [ ] 🍎 Интеграция с контекстным меню macOS (Finder/Automator)
- [ ] 🤖 Прямая отправка в OpenAI/Anthropic API (без буфера обмена)
- [ ] 🏛️ Анализ Hexagonal Architecture
- [ ] 🔌 Плагинная система для анализаторов и экспортёров
- [ ] 🌐 i18n — переключение языка интерфейса внутри приложения

---

## 👨‍💻 Команда

**Разработчик:** mcniki · [VK: gor_niki](https://vk.com/gor_niki) · Issues & PRs на GitHub

---

## 🤝 Как помочь проекту?

1. **Форкните** репозиторий
2. Создайте ветку: `git checkout -b feature/AmazingFeature`
3. Закоммитьте: `git commit -m 'Add AmazingFeature'`
4. Запушьте: `git push origin feature/AmazingFeature`
5. Откройте **Pull Request**

Пожалуйста, соблюдайте SOLID-принципы и архитектурные паттерны (см. `docs/ARCHITECTURE.md`).

---

## 📄 Лицензия

Проект распространяется под лицензией MIT. Подробности — в файле `LICENSE`.

</div>

<!-- ==================== ENGLISH ==================== -->

<div class="lang-en">

## 🌟 About

**CodeContext AI** is a powerful desktop tool for preparing your codebase to work with Large Language Models (LLMs). It scans project folders, analyzes structure, builds dependency graphs, and generates a single, perfectly structured prompt — optimized for token consumption and architectural clarity.

### ❓ Why?

When working with AI, developers face context window token limits — LLMs "lose" architectural coherence when code is copied in parts. **CodeContext AI solves this**: collect your entire project into one structured prompt in a few clicks, saving up to 80% on tokens.

---

## 🚀 Features

| Feature | CodeContext AI | Manual |
|---|---|---|
| 🗜️ Minify + Skeleton | **Up to 80%** token reduction | Manual copy-paste |
| 🧩 LLM Patcher | Preview & apply JSON patches | Not available |
| ✅ LLM Checker | Auto-verify code before saving | Not available |
| 🔗 AST dependency graph | Python, JS/TS, Vue | File listing only |
| 🖱️ Context menu | Windows / Linux | None |
| 🎨 Themes | Apple, Modern, custom JSON | Fixed UI |
| ⚙️ UI customization (v1.14+) | Premiere Pro-style | Fixed UI |

---

## 📥 Installation

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

### Arch Linux (AUR)

```bash
yay -S codecontext-ai
```

### Windows .exe

```bash
pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py
```

---

## 💻 GUI Mode

```bash
python main.py
```

### 1. Interface Overview

The window is split into three zones:
- **Left sidebar (tabs)** — scan settings, filters, prompts, LLM config, themes
- **Center area** — folder list, file tree, token analytics
- **Top action bar** — Minify/No Comments/Skeleton toggles, output format, action buttons

### 2. Adding a Project

| Action | How |
|---|---|
| Drag & drop | Just drag a project folder into the window |
| Browse dialog | Click "+ Папка ПК" on the **Sources** tab |
| GitHub repo | Click "+ GitHub / PR" — paste a repository or Pull Request URL |
| Save config | Click "💾 Save config" — creates `.codecontextrc` for reuse |

**GitHub loading modes:**
- **Save permanently** — clones the repo to a folder on your disk
- **Temporary** — clones to a temp folder (deleted on app close)

### 3. Scan Configuration

#### Sources Tab

| Option | Description |
|---|---|
| ☑ Git Changes Only | Include only files changed in the last commit |
| ☑ Respect .gitignore | Auto-exclude files from `.gitignore` |
| 🔍 Scan Files | Build the file tree with metadata |

#### Filters Tab

| Option | Description |
|---|---|
| **Extension presets** | Quick switch between language sets (Python, Web, Golang, Rust, C#, etc.) |
| **Extensions** | Custom file extension whitelist (space-separated or one per line) |
| **Ignored paths** | Skip these folders/files (node_modules, .git, build, dist, etc.) |
| ☑ Include file tree | Prepends folder structure to the prompt |
| ☑ Include dependency map | AST-based import analysis for Python/JS/TS |
| ☑ Include Mermaid graph | Generates an architecture diagram in Mermaid format |

**Saving custom presets:** configure filters, click 💾, enter a name. It appears in the dropdown.

#### Prompts Tab

| Option | Description |
|---|---|
| **Prompt presets** | Quick change of system prompt (Code Review, Bug Hunter, Refactoring, Security Audit, etc.) |
| **System prompt** | Custom prompt editor. This text is sent to the LLM as system context |
| **🧩 Apply JSON patch** | Paste an LLM JSON response with code changes — preview diff and apply to disk |

**Using JSON patches:**
1. Ask the LLM to return a JSON array of changes:
   ```json
   [
     {"action": "replace", "file": "main.py", "search": "def old():", "content": "def new():"}
   ]
   ```
2. Paste the JSON and click **"Next"**
3. **Safety Diff Viewer** opens — shows BEFORE/AFTER for each file
4. Check/uncheck files you want to apply
5. Optionally click **"🤖 Check via LLM"** — the neural net validates the patch
6. If the LLM suggests an improved version, a diff panel appears comparing your patch vs the AI version
7. Click **"💾 Save selected to disk"**

### 4. Output Format Settings

Action Panel (top bar):

| Option | Description |
|---|---|
| ☑ Minify | Strips extra whitespace and blank lines |
| ☑ No Comments | Removes all comments from the code |
| ☑ No Secrets | Masks potential secrets (API keys, passwords, tokens) |
| ☑ Skeleton ☠️ | **Strips function bodies**, keeping only names and class structure — maximum token savings |
| Format | Markdown (default), XML, Plain, JSONL Chunks, Custom (Jinja2) |
| 📁 (template) | Select a Jinja2 template (active when format is Custom) |

**Skeleton Mode details:**
- Removes function implementations, keeps `def func_name(...):`
- Preserves all classes with methods and fields (but no method bodies)
- Lets the LLM "understand" a massive project's architecture with minimal tokens
- Example: `def calculate_total(price, tax):` → `# ... implementation ...`

### 5. Action Buttons

| Button | Action |
|---|---|
| 👀 Preview | Opens **Advanced Preview Dialog** with two tabs: "Final Prompt" and "Before/After" |
| 📋 Copy to Clipboard | Copies to clipboard — paste into ChatGPT / Claude |
| 🚀 Send to ChatGPT / Claude | Opens the web chat and pastes context automatically |
| 💻 Open in Editor | Opens in VS Code / Cursor (configurable in LLM & OS tab) |
| 💾 Save to File | Saves result to a file on disk |

### 6. Advanced Preview Dialog

**"📝 Final Prompt" tab:**
- **Left:** list of files in the prompt — click to jump to that file in the text
- **Right:** full prompt text with syntax highlighting (Markdown/XML)
- **Buttons:** "📋 Copy All" / "✂️ Copy This File Only"

**"🔍 Before/After (Optimization)" tab:**
- Select a file from the dropdown
- Shows a colored diff between original and optimized version
- Token savings counter: `Before: 1500 → After: 300 (Compressed 80.0%)`

### 7. LLM & OS Settings Tab

#### LLM Checker

| Option | Description |
|---|---|
| ☑ Enable verification | Activates automatic LLM patch verification before applying |
| URL | API endpoint (default `https://api.openai.com/v1`) |
| Key | API key (persisted in settings) |
| Model | Model name (`gpt-4o-mini`, `claude-3-haiku`, `llama3`, etc.) |

**Quick presets:**
- 🦙 **Ollama** — `http://localhost:11434/v1`, model `llama3`
- 🖥 **LM Studio** — `http://localhost:1234/v1`, model `local-model`

#### OS Integration

| Button | Description |
|---|---|
| Install context menu | Adds "Open with CodeContext AI" to the right-click menu |
| Remove context menu | Removes the context menu entry |
| Add to PATH | Installs the global `codecontext` CLI command |
| Remove from PATH | Removes the global CLI command |
| **Editor** | Specify editor: `code`, `cursor`, `idea`, `vim` (blank = system default) |

#### Updates

| Option | Description |
|---|---|
| ☑ Receive pre-releases | Opt in to pre-release versions |
| 🔄 Check updates | Check for new version and download/install automatically |

### 8. Themes Tab

| Option | Description |
|---|---|
| **Theme** | Select a theme (Apple, Modern) |
| **Mode** | Light / Dark |
| 📂 Open themes folder | Opens the folder where you can add custom JSON themes |
| ➕ Import theme | Load a `.json` theme from a file |

Imported themes appear in the dropdown automatically.

### 9. Token Analytics

The **"📊 Token Analytics"** tab shows a table of scanned files:
- File path
- Token count (via `tiktoken`, OpenAI algorithms)
- Post-compression size
- Savings percentage
- Cost in USD for the selected LLM model (prices auto-update from server)

### 10. 🎛️ UI Customization (v1.14+)

Click **⚙** next to the version label in the sidebar — the "Interface Settings (Premiere Pro style)" dialog opens.

**Toggle sidebar tabs:**
- 📡 Sources · 🎯 Filters · 📝 Prompts · LLM & OS · 🎨 Themes

**Toggle action buttons:**
- 👀 Preview · 📋 Clipboard · 🚀 ChatGPT · 💻 Editor · 💾 File

Changes are saved and applied immediately.

### 11. Command Palette

`Ctrl+Shift+P` — mouse-free access to all actions:

| Command | Description |
|---|---|
| Generate: Copy to clipboard | Process and copy result to clipboard |
| Generate: Open in Editor | Process and open in VS Code / Cursor |
| Generate: Preview | Open Advanced Preview Dialog |
| Options: Toggle Minify / Skeleton / Mermaid | Toggle corresponding options |
| Actions: Apply JSON patch | Open JSON patch dialog |
| Settings: Toggle Dark/Light theme | Switch between light and dark mode |
| System: Check for updates | Check for new version |

---

## 💻 CLI Mode

```bash
python main.py --cli --path /path/to/project [options]
```

### Full parameter reference

```bash
python main.py --help
```

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| `--cli` | flag | Enable CLI mode (no GUI) | `--cli` |
| `--path` | list | Project path(s) | `--path ./app ./lib` |
| `--ext` | str | Extensions (space-separated) | `--ext ".py .js .ts"` |
| `--ignore` | str | Ignored paths (comma-separated) | `--ignore "node_modules,.git"` |
| `--mode` | enum | Dependency mode: `none`, `default`, `shallow`, `deep` | `--mode deep` |
| `--format` | enum | Output format: `markdown`, `xml`, `plain`, `jsonl_chunk` | `--format xml` |
| `--minify` | flag | Enable minification | `--minify` |
| `--no-comments` | flag | Strip comments | `--no-comments` |
| `--no-secrets` | flag | Mask secrets | `--no-secrets` |
| `--skeleton` | flag | Skeleton mode | `--skeleton` |
| `--output` | str | Output file path | `--output result.txt` |
| `--stdout` | flag | Print result to stdout | `--stdout` |
| `--git` | flag | Changed files only (Git) | `--git` |
| `--gitignore` | flag | Respect .gitignore | `--gitignore` |
| `--tree` | flag | Include file tree | `--tree` |
| `--mermaid` | flag | Include Mermaid graph | `--mermaid` |
| `--dependencies` | flag | Include dependency map | `--dependencies` |
| `--patch` | str | Apply LLM JSON patch | `--patch patch.json` |
| `--template` | str | Jinja2 template path | `--template my.j2` |
| `--system-prompt` | str | Custom system prompt | `--system-prompt "Review"` |
| `--no-banner` | flag | Suppress startup banner | `--no-banner` |

### Usage examples

```bash
# Minimal run — output to stdout
python main.py --cli --path ./myapp --stdout

# Full analysis with XML output to file
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff mode — only changed files
python main.py --cli --path ./myapp --git --gitignore --stdout

# Apply LLM JSON patch
python main.py --cli --path ./myapp --patch llm_response.json

# Custom Jinja2 template
python main.py --cli --path ./myapp --template custom_template.j2 --stdout

# Deep analysis with Mermaid diagram
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Pipe to clipboard (Linux)
python main.py --cli --path ./myapp --minify --no-comments --stdout | xclip -selection clipboard

# Using global CLI command (Linux)
codecontext --cli --path ./myapp --stdout

# Multiple project paths
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml
```

---

## 🏗️ Tech Stack

| Component | Technology |
|---|---|
| Language | Python 3.10+ |
| GUI Framework | PySide6 (Qt 6) |
| Architecture | Clean Architecture + Redux-like (Store → Controller → UI) |
| Tokenization | `tiktoken` (OpenAI) |
| Templating | `jinja2` (11 built-in templates) |
| Async | `asyncio` |
| AST parsers | `ast` (Python), `tree-sitter` (JS/TS/Go/Rust) |
| Distribution | PyInstaller (Windows .exe), AUR (Arch Linux) |

---

## 📁 Project Structure

```
CodeContext/
├── main.py                  # Application entry point
├── VERSION.txt              # Read at startup for version display
├── requirements.txt         # Python dependencies
├── assets/
│   └── images/logo.png      # App logo
├── aur_build/               # AUR package for Arch Linux
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # Built-in JSON theme files
└── src/
    ├── main_app.py          # Bootstrap: init Store, Controller, Window
    ├── store/               # Redux-like state management
    │   ├── state.py         # Data classes: AppSettings, AppState, ProcessedFile
    │   └── store.py         # Store with subscribe/dispatch pattern
    ├── controllers/         # Business logic & orchestration
    │   └── main_controller.py
    ├── ui/                  # PySide6 layer (all visual components)
    │   ├── main_window.py   # Main window with QSplitter layout
    │   ├── dialogs.py       # All modal dialogs
    │   ├── theme_manager.py # Theme load/apply engine
    │   └── components/
    │       ├── sidebar.py       # Left panel (5 settings tabs)
    │       ├── action_panel.py  # Top bar (checkboxes + action buttons)
    │       ├── folder_list.py   # Added folders list
    │       ├── file_tree.py     # File tree with exclusion checkboxes
    │       ├── log_panel.py     # Log panel (bottom-left)
    │       ├── status_bar.py    # Status bar (tokens, cost, progress)
    │       ├── empty_state.py   # Empty state with Drag & Drop hint
    │       └── analytics_panel.py # Token & cost analytics table
    └── utils/
        ├── config.py        # Presets, version, paths, PricingManager
        ├── async_runtime.py # asyncio → Qt event loop bridge
        └── ...
```

---

## 🗺️ Roadmap

- [ ] 🍎 macOS Finder context menu (Automator)
- [ ] 🤖 Direct OpenAI/Anthropic API integration (no clipboard)
- [ ] 🏛️ Hexagonal Architecture analysis strategy
- [ ] 🔌 Plugin system for custom analyzers and exporters
- [ ] 🌐 i18n — in-app language switching

---

## 👨‍💻 Team

**Developer:** mcniki · [VK: gor_niki](https://vk.com/gor_niki) · Issues & PRs on GitHub

---

## 🤝 Contributing

1. **Fork** the repository
2. Create a feature branch: `git checkout -b feature/AmazingFeature`
3. Commit: `git commit -m 'Add AmazingFeature'`
4. Push: `git push origin feature/AmazingFeature`
5. Open a **Pull Request**

Please follow SOLID principles and project architectural patterns (see `docs/ARCHITECTURE.md`).

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for details.

</div>

<style>
  .lang-label {
    display: inline-block;
    padding: 8px 28px;
    margin: 0 6px;
    cursor: pointer;
    border-radius: 20px;
    font-size: 15px;
    font-weight: 600;
    transition: all 0.25s ease;
    border: 2px solid #d0d7de;
    background: #f6f8fa;
    color: #656d76;
    user-select: none;
  }
  .lang-label:hover {
    transform: translateY(-1px);
    box-shadow: 0 3px 10px rgba(0,0,0,0.12);
    border-color: #0969da;
    color: #0969da;
  }
  .lang-ru, .lang-en {
    display: none;
  }
  #lang_ru:checked ~ div[align="center"] label[for="lang_ru"] {
    background: #0969da;
    border-color: #0969da;
    color: #fff;
    box-shadow: 0 3px 10px rgba(9,105,218,0.35);
  }
  #lang_en:checked ~ div[align="center"] label[for="lang_en"] {
    background: #0969da;
    border-color: #0969da;
    color: #fff;
    box-shadow: 0 3px 10px rgba(9,105,218,0.35);
  }
  #lang_ru:checked ~ .lang-ru {
    display: block;
  }
  #lang_en:checked ~ .lang-en {
    display: block;
  }
</style>
