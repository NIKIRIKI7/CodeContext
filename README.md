<input type="radio" name="lang" id="lang_ru" checked hidden>
<input type="radio" name="lang" id="lang_en" hidden>

<div align="center" class="lang-header">

<label for="lang_ru" class="lang-btn">🇷🇺 Русский</label>
<label for="lang_en" class="lang-btn">🇬🇧 English</label>

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

<h2>🌟 О проекте</h2>

<p><b>CodeContext AI</b> — мощный десктопный инструмент для подготовки кодовой базы к работе с большими языковыми моделями (LLM). Сканирует папки проекта, анализирует структуру, строит граф зависимостей и генерирует единый структурированный промпт.</p>

<h3>❓ Зачем?</h3>
<p>При работе с ИИ разработчики упираются в лимит токенов — нейросеть «теряет» архитектуру проекта, когда код копируется частями. <b>CodeContext AI решает это</b>: в пару кликов собирает ВЕСЬ проект в один промпт, экономя до 80% токенов.</p>

<hr>

<h2>🚀 Возможности</h2>

<table>
<thead><tr><th>Возможность</th><th>CodeContext AI</th><th>Вручную</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify + Skeleton</td><td><b>До 80%</b> экономии токенов</td><td>Копировать вручную</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Предпросмотр и JSON-патчи</td><td>Нет</td></tr>
<tr><td>✅ LLM Checker</td><td>Авто-проверка кода перед записью</td><td>Нет</td></tr>
<tr><td>🔗 AST-граф</td><td>Python, JS/TS, Vue</td><td>Только файлы</td></tr>
<tr><td>🖱️ Контекстное меню</td><td>Windows / Linux</td><td>Нет</td></tr>
<tr><td>🎨 Темы</td><td>Apple, Modern, кастомные</td><td>Фиксированный UI</td></tr>
<tr><td>⚙️ Кастомизация (v1.14+)</td><td>Premiere Pro-style</td><td>Фиксированный UI</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Установка</h2>

<p><b>Требования:</b> Python 3.10+, Git</p>

<pre>git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext
python -m venv venv
# Windows:
venv\Scripts\activate
# Linux/macOS:
source venv/bin/activate
pip install -r requirements.txt</pre>

<h3>Arch Linux (AUR)</h3>
<pre>yay -S codecontext-ai</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<hr>

<h2>💻 Работа в GUI-режиме</h2>
<pre>python main.py</pre>

<h3>1. Обзор интерфейса</h3>
<p>Окно разделено на три зоны:</p>
<ul>
<li><b>Левая панель (вкладки)</b> — настройки сканирования, фильтры, промпты, LLM, темы</li>
<li><b>Центральная область</b> — список папок, дерево файлов, аналитика токенов</li>
<li><b>Верхняя панель (Action Panel)</b> — опции Minify/No Comments/Skeleton и кнопки действий</li>
</ul>

<h3>2. Добавление проекта</h3>
<table>
<thead><tr><th>Действие</th><th>Как сделать</th></tr></thead>
<tbody>
<tr><td>Перетащить папку</td><td>Просто перетащите папку проекта в окно</td></tr>
<tr><td>Выбрать через диалог</td><td>Кнопка «+ Папка ПК» на вкладке <b>Источники</b></td></tr>
<tr><td>GitHub репозиторий</td><td>Кнопка «+ GitHub / PR» — вставьте URL репозитория или Pull Request</td></tr>
<tr><td>Сохранить конфиг</td><td>Кнопка «💾 Сохранить конфиг» — создаст <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>Режимы загрузки GitHub:</b></p>
<ul>
<li><b>Сохранить навсегда</b> — клонирует репозиторий в выбранную папку на диске</li>
<li><b>Временная загрузка</b> — клонирует во временную папку (удаляется при закрытии программы)</li>
</ul>

<h3>3. Настройка сканирования</h3>

<h4>Вкладка «📡 Источники»</h4>
<table>
<thead><tr><th>Опция</th><th>Описание</th></tr></thead>
<tbody>
<tr><td>☑ Только Git Changes</td><td>Включить в результат только файлы, изменённые в последнем коммите</td></tr>
<tr><td>☑ Учитывать .gitignore</td><td>Автоматически исключать файлы из <code>.gitignore</code></td></tr>
<tr><td>🔍 Сканировать файлы</td><td>Запустить сканирование — построить дерево файлов с метаданными</td></tr>
</tbody>
</table>

<h4>Вкладка «🎯 Фильтры»</h4>
<table>
<thead><tr><th>Опция</th><th>Описание</th></tr></thead>
<tbody>
<tr><td><b>Пресеты расширений</b></td><td>Быстрое переключение между наборами расширений (Python, Web, Golang, Rust, C# и др.)</td></tr>
<tr><td><b>Расширения</b></td><td>Кастомный список расширений файлов для включения</td></tr>
<tr><td><b>Игнорировать пути</b></td><td>Список папок/файлов для исключения (node_modules, .git, build, dist и т.д.)</td></tr>
<tr><td>☑ Включить дерево файлов</td><td>Добавляет структуру папок в начало промпта</td></tr>
<tr><td>☑ Включить карту зависимостей</td><td>AST-анализ импортов для Python/JS/TS — показывает связи между файлами</td></tr>
<tr><td>☑ Включить Mermaid-граф</td><td>Генерирует архитектурную диаграмму в формате Mermaid</td></tr>
</tbody>
</table>

<p>💡 <b>Сохранение кастомных пресетов:</b> настройте фильтры, нажмите 💾, введите имя — пресет появится в выпадающем списке.</p>

<h4>Вкладка «📝 Промпты»</h4>
<table>
<thead><tr><th>Опция</th><th>Описание</th></tr></thead>
<tbody>
<tr><td><b>Пресеты промптов</b></td><td>Быстрая смена системного промпта (Code Review, Bug Hunter, Refactoring, Security Audit и др.)</td></tr>
<tr><td><b>Системный промпт</b></td><td>Текстовое поле для кастомного промпта. Именно этот текст будет отправлен LLM как system-контекст</td></tr>
<tr><td><b>🧩 Применить JSON-патч</b></td><td>Вставьте JSON-ответ от LLM с изменениями — программа покажет Diff и даст применить файлы на диск</td></tr>
</tbody>
</table>

<p><b>Использование JSON-патча:</b></p>
<ol>
<li>Попросите LLM вернуть JSON-массив с изменениями: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Вставьте JSON в диалог и нажмите <b>«Далее»</b></li>
<li>Откроется <b>Safety Diff Viewer</b> — для каждого файла показаны изменения ДО/ПОСЛЕ</li>
<li>Отметьте нужные файлы галочками (или снимите, если изменение некорректно)</li>
<li>Нажмите <b>«🤖 Проверить через LLM»</b> — нейросеть проверит патч на ошибки</li>
<li>Если LLM предлагает улучшенную версию — появится панель с Diff между вашим патчем и вариантом ИИ</li>
<li>Нажмите <b>«💾 Сохранить выбранные на диск»</b></li>
</ol>

<h3>4. Настройка формата вывода</h3>
<table>
<thead><tr><th>Опция</th><th>Описание</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>Удаляет лишние пробелы и пустые строки</td></tr>
<tr><td>☑ No Comments</td><td>Вырезает все комментарии из кода</td></tr>
<tr><td>☑ No Secrets</td><td>Маскирует потенциальные секреты (ключи API, пароли, токены)</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Удаляет тела функций</b>, оставляя только названия и структуру классов — макс. экономия токенов</td></tr>
<tr><td>Формат</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 шаблон</td><td>Выбор Jinja2-шаблона (активно при формате Custom)</td></tr>
</tbody>
</table>

<p><b>Skeleton Mode:</b> удаляет реализацию функций (<code>def func_name(...):  # ... implementation ...</code>), оставляя все классы и методы — LLM видит архитектуру гигантского проекта, тратя минимум токенов.</p>

<h3>5. Действия с результатом</h3>
<table>
<thead><tr><th>Кнопка</th><th>Действие</th></tr></thead>
<tbody>
<tr><td>👀 Предпросмотр</td><td>Открывает <b>Advanced Preview Dialog</b> (вкладки: «Итоговый промпт» + «До/После»)</td></tr>
<tr><td>📋 В Буфер обмена</td><td>Копирует результат в буфер обмена — вставьте в ChatGPT / Claude</td></tr>
<tr><td>🚀 Отправить в ChatGPT / Claude</td><td>Автоматически открывает веб-версию чата и вставляет контекст</td></tr>
<tr><td>💻 В редактор</td><td>Открывает результат в VS Code / Cursor (настраивается в LLM & ОС)</td></tr>
<tr><td>💾 В Файл</td><td>Сохраняет результат в файл на диске</td></tr>
</tbody>
</table>

<h3>6. Advanced Preview Dialog</h3>
<p><b>Вкладка «📝 Итоговый промпт»:</b> слева — список файлов, справа — полный текст с подсветкой. Кнопки «📋 Копировать всё» / «✂️ Скопировать только этот файл».</p>
<p><b>Вкладка «🔍 До/После»:</b> выберите файл — цветной Diff между оригиналом и оптимизацией. Счётчик: <code>Было: 1500 → Стало: 300 (80%)</code>.</p>

<h3>7. LLM & ОС</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Включить проверку</td><td>Автоматическая LLM-верификация патчей перед применением</td></tr>
<tr><td>URL / Ключ / Модель</td><td>API эндпоинт (по умолч. OpenAI), ключ, модель</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">Интеграция с ОС</th></tr></thead>
<tbody>
<tr><td>В меню проводника</td><td>Добавляет «Open with CodeContext AI» в контекстное меню</td></tr>
<tr><td>Добавить в PATH</td><td>Глобальная команда <code>codecontext</code> из терминала</td></tr>
<tr><td>Редактор</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Темы</h3>
<ul>
<li><b>Тема:</b> Apple, Modern — <b>Режим:</b> светлая / тёмная</li>
<li>📂 Открыть папку тем / ➕ Импортировать тему (.json)</li>
</ul>

<h3>9. 📊 Аналитика токенов</h3>
<p>Таблица со сканированными файлами: путь, токены (tiktoken), сжатие, экономия %, стоимость в $ для модели.</p>

<h3>10. 🎛️ Кастомизация интерфейса (v1.14+)</h3>
<p>Нажмите <b>⚙</b> рядом с версией — диалог «Настройка интерфейса (Premiere Pro style)». Включайте/выключайте вкладки (Источники, Фильтры, Промпты, LLM & ОС, Темы) и кнопки действий (Предпросмотр, Буфер, ChatGPT, Редактор, Файл).</p>

<h3>11. Командная палитра</h3>
<p><code>Ctrl+Shift+P</code> — быстрый доступ ко всем действиям без мыши.</p>

<hr>

<h2>💻 Работа в CLI-режиме</h2>
<pre>python main.py --cli --path /путь/к/проекту [опции]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Параметр</th><th>Тип</th><th>Описание</th><th>Пример</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>флаг</td><td>CLI-режим (без GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>сп.</td><td>Путь к проекту</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>стр.</td><td>Расширения</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>стр.</td><td>Игнорируемые пути</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>выб.</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>выб.</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>флаг</td><td>Minify</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>флаг</td><td>Без комментариев</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>флаг</td><td>Без секретов</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>флаг</td><td>Skeleton-режим</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>стр.</td><td>Файл результата</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>флаг</td><td>Вывод в консоль</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>флаг</td><td>Только Git-изменения</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>флаг</td><td>Учитывать .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>флаг</td><td>Дерево файлов</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>флаг</td><td>Mermaid-граф</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>флаг</td><td>Карта зависимостей</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>стр.</td><td>JSON-патч от LLM</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>стр.</td><td>Jinja2-шаблон</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>стр.</td><td>Кастомный промпт</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Примеры</h3>
<pre># Минимальный запуск
python main.py --cli --path ./myapp --stdout

# Полный анализ с XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# JSON-патч
python main.py --cli --path ./myapp --patch llm_response.json

# Кастомный шаблон
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid-диаграмма
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Несколько путей
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Стек технологий</h2>
<table>
<thead><tr><th>Компонент</th><th>Технология</th></tr></thead>
<tbody>
<tr><td>Язык</td><td>Python 3.10+</td></tr>
<tr><td>GUI</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Архитектура</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Токенизация</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Шаблоны</td><td>jinja2 (11 встроенных)</td></tr>
<tr><td>AST-парсеры</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Дистрибуция</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>📁 Структура проекта</h2>
<pre>CodeContext/
├── main.py                  # Точка входа
├── VERSION.txt              # Версия
├── requirements.txt         # Зависимости
├── assets/images/logo.png   # Логотип
├── aur_build/               # AUR-пакет
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # JSON-темы
└── src/
    ├── main_app.py          # Загрузчик
    ├── store/               # Redux-like (state.py, store.py)
    ├── controllers/         # Бизнес-логика
    ├── ui/                  # PySide6
    │   ├── main_window.py
    │   ├── dialogs.py
    │   ├── theme_manager.py
    │   └── components/
    │       ├── sidebar.py, action_panel.py, folder_list.py
    │       ├── file_tree.py, log_panel.py, status_bar.py
    │       ├── empty_state.py, analytics_panel.py
    └── utils/
        ├── config.py, async_runtime.py</pre>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>🍎 macOS Finder context menu</li>
<li>🤖 Прямая отправка в OpenAI/Anthropic API</li>
<li>🏛️ Hexagonal Architecture анализ</li>
<li>🔌 Плагинная система</li>
<li>🌐 i18n интерфейса</li>
</ul>

<hr>

<h2>👨‍💻 Команда</h2>
<p><b>Разработчик:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs на GitHub</p>

<hr>

<h2>🤝 Как помочь?</h2>
<ol>
<li>Форкните репозиторий</li>
<li>Ветка: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Коммит: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Пуш: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Соблюдайте SOLID-принципы (см. <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 Лицензия</h2>
<p>MIT. Подробности в <code>LICENSE</code>.</p>

</div>

<!-- ==================== ENGLISH ==================== -->

<div class="lang-en">

<h2>🌟 About</h2>

<p><b>CodeContext AI</b> is a powerful desktop tool for preparing your codebase to work with Large Language Models (LLMs). It scans project folders, analyzes structure, builds dependency graphs, and generates a single, perfectly structured prompt — optimized for token consumption and architectural clarity.</p>

<h3>❓ Why?</h3>
<p>When working with AI, developers face context window token limits — LLMs "lose" architectural coherence when code is copied in parts. <b>CodeContext AI solves this</b>: collect your entire project into one structured prompt in a few clicks, saving up to 80% on tokens.</p>

<hr>

<h2>🚀 Features</h2>

<table>
<thead><tr><th>Feature</th><th>CodeContext AI</th><th>Manual</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify + Skeleton</td><td><b>Up to 80%</b> token reduction</td><td>Manual copy-paste</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Preview & apply JSON patches</td><td>Not available</td></tr>
<tr><td>✅ LLM Checker</td><td>Auto-verify code before saving</td><td>Not available</td></tr>
<tr><td>🔗 AST dependency graph</td><td>Python, JS/TS, Vue</td><td>File listing only</td></tr>
<tr><td>🖱️ Context menu</td><td>Windows / Linux</td><td>None</td></tr>
<tr><td>🎨 Themes</td><td>Apple, Modern, custom JSON</td><td>Fixed UI</td></tr>
<tr><td>⚙️ UI customization (v1.14+)</td><td>Premiere Pro-style</td><td>Fixed UI</td></tr>
</tbody>
</table>

<hr>

<h2>📥 Installation</h2>

<p><b>Prerequisites:</b> Python 3.10+, Git</p>

<pre>git clone https://github.com/NIKIRIKI7/CodeContext.git
cd CodeContext
python -m venv venv
# Windows:
venv\Scripts\activate
# Linux/macOS:
source venv/bin/activate
pip install -r requirements.txt</pre>

<h3>Arch Linux (AUR)</h3>
<pre>yay -S codecontext-ai</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<hr>

<h2>💻 GUI Mode</h2>
<pre>python main.py</pre>

<h3>1. Interface Overview</h3>
<p>The window is split into three zones:</p>
<ul>
<li><b>Left sidebar (tabs)</b> — scan settings, filters, prompts, LLM config, themes</li>
<li><b>Center area</b> — folder list, file tree, token analytics</li>
<li><b>Top action bar</b> — Minify/No Comments/Skeleton toggles, output format, action buttons</li>
</ul>

<h3>2. Adding a Project</h3>
<table>
<thead><tr><th>Action</th><th>How</th></tr></thead>
<tbody>
<tr><td>Drag & drop</td><td>Just drag a project folder into the window</td></tr>
<tr><td>Browse dialog</td><td>Click "+ Папка ПК" on the <b>Sources</b> tab</td></tr>
<tr><td>GitHub repo</td><td>Click "+ GitHub / PR" — paste a repo or Pull Request URL</td></tr>
<tr><td>Save config</td><td>Click "💾 Save config" — creates <code>.codecontextrc</code></td></tr>
</tbody>
</table>

<p><b>GitHub loading modes:</b></p>
<ul>
<li><b>Save permanently</b> — clones to a folder on your disk</li>
<li><b>Temporary</b> — clones to a temp folder (deleted on app close)</li>
</ul>

<h3>3. Scan Configuration</h3>

<h4>Sources Tab</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td>☑ Git Changes Only</td><td>Include only files changed in the last commit</td></tr>
<tr><td>☑ Respect .gitignore</td><td>Auto-exclude files from <code>.gitignore</code></td></tr>
<tr><td>🔍 Scan Files</td><td>Build the file tree with metadata</td></tr>
</tbody>
</table>

<h4>Filters Tab</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td><b>Extension presets</b></td><td>Quick switch between language sets (Python, Web, Golang, Rust, C#, etc.)</td></tr>
<tr><td><b>Extensions</b></td><td>Custom file extension whitelist</td></tr>
<tr><td><b>Ignored paths</b></td><td>Skip folders/files (node_modules, .git, build, dist, etc.)</td></tr>
<tr><td>☑ Include file tree</td><td>Prepends folder structure to the prompt</td></tr>
<tr><td>☑ Include dependency map</td><td>AST-based import analysis for Python/JS/TS</td></tr>
<tr><td>☑ Include Mermaid graph</td><td>Architecture diagram in Mermaid format</td></tr>
</tbody>
</table>

<p>💡 <b>Saving custom presets:</b> configure filters, click 💾, enter a name.</p>

<h4>Prompts Tab</h4>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td><b>Prompt presets</b></td><td>Quick change of system prompt (Code Review, Bug Hunter, Refactoring, etc.)</td></tr>
<tr><td><b>System prompt</b></td><td>Custom prompt — sent to LLM as system context</td></tr>
<tr><td><b>🧩 Apply JSON patch</b></td><td>Paste LLM JSON response — preview diff and apply to disk</td></tr>
</tbody>
</table>

<p><b>Using JSON patches:</b></p>
<ol>
<li>Ask LLM for a JSON array: <code>[{"action": "replace", "file": "main.py", "search": "...", "content": "..."}]</code></li>
<li>Paste JSON, click <b>"Next"</b> → <b>Safety Diff Viewer</b> opens</li>
<li>Check/uncheck files, optionally click <b>"🤖 Check via LLM"</b></li>
<li>Click <b>"💾 Save selected to disk"</b></li>
</ol>

<h3>4. Output Format Settings</h3>
<table>
<thead><tr><th>Option</th><th>Description</th></tr></thead>
<tbody>
<tr><td>☑ Minify</td><td>Strips whitespace and blank lines</td></tr>
<tr><td>☑ No Comments</td><td>Removes all comments</td></tr>
<tr><td>☑ No Secrets</td><td>Masks API keys, passwords, tokens</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Strips function bodies</b> — maximum token savings</td></tr>
<tr><td>Format</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 template</td><td>Jinja2 template picker</td></tr>
</tbody>
</table>

<p><b>Skeleton Mode:</b> removes function implementations (<code>def func_name(...):  # ... implementation ...</code>), preserving all classes — lets LLM understand massive projects with minimal tokens.</p>

<h3>5. Action Buttons</h3>
<table>
<thead><tr><th>Button</th><th>Action</th></tr></thead>
<tbody>
<tr><td>👀 Preview</td><td><b>Advanced Preview Dialog</b> — "Final Prompt" + "Before/After" tabs</td></tr>
<tr><td>📋 Copy to Clipboard</td><td>Copy result — paste into ChatGPT / Claude</td></tr>
<tr><td>🚀 Send to ChatGPT / Claude</td><td>Opens web chat and pastes context</td></tr>
<tr><td>💻 Open in Editor</td><td>Opens in VS Code / Cursor</td></tr>
<tr><td>💾 Save to File</td><td>Save result to disk</td></tr>
</tbody>
</table>

<h3>6. Advanced Preview Dialog</h3>
<p><b>"📝 Final Prompt" tab:</b> file list (left) + full text with highlighting (right). Copy All / Copy File.</p>
<p><b>"🔍 Before/After" tab:</b> colored diff between original and optimized. Counter: <code>Before: 1500 → After: 300 (80%)</code>.</p>

<h3>7. LLM & OS</h3>
<table>
<thead><tr><th colspan="2">LLM Checker</th></tr></thead>
<tbody>
<tr><td>☑ Enable verification</td><td>Auto LLM patch verification before applying</td></tr>
<tr><td>URL / Key / Model</td><td>API endpoint (default OpenAI), key, model</td></tr>
<tr><td>🦙 Ollama</td><td><code>http://localhost:11434/v1</code> / <code>llama3</code></td></tr>
<tr><td>🖥 LM Studio</td><td><code>http://localhost:1234/v1</code> / <code>local-model</code></td></tr>
</tbody>
</table>

<table>
<thead><tr><th colspan="2">OS Integration</th></tr></thead>
<tbody>
<tr><td>Install context menu</td><td>"Open with CodeContext AI" in right-click menu</td></tr>
<tr><td>Add to PATH</td><td>Global <code>codecontext</code> CLI command</td></tr>
<tr><td>Editor</td><td><code>code</code>, <code>cursor</code>, <code>idea</code>, <code>vim</code></td></tr>
</tbody>
</table>

<h3>8. Themes</h3>
<ul>
<li><b>Theme:</b> Apple, Modern — <b>Mode:</b> light / dark</li>
<li>📂 Open themes folder / ➕ Import theme (.json)</li>
</ul>

<h3>9. 📊 Token Analytics</h3>
<p>Table: file path, tokens (tiktoken), compression, savings %, cost for model.</p>

<h3>10. 🎛️ UI Customization (v1.14+)</h3>
<p>Click <b>⚙</b> next to version — "Interface Settings (Premiere Pro style)" dialog. Toggle tabs (Sources, Filters, Prompts, LLM & OS, Themes) and action buttons (Preview, Clipboard, ChatGPT, Editor, File).</p>

<h3>11. Command Palette</h3>
<p><code>Ctrl+Shift+P</code> — mouse-free access to all actions.</p>

<hr>

<h2>💻 CLI Mode</h2>
<pre>python main.py --cli --path /path/to/project [options]</pre>
<pre>python main.py --help</pre>

<table>
<thead><tr><th>Parameter</th><th>Type</th><th>Description</th><th>Example</th></tr></thead>
<tbody>
<tr><td><code>--cli</code></td><td>flag</td><td>CLI mode (no GUI)</td><td><code>--cli</code></td></tr>
<tr><td><code>--path</code></td><td>list</td><td>Project path</td><td><code>--path ./app</code></td></tr>
<tr><td><code>--ext</code></td><td>str</td><td>Extensions</td><td><code>--ext ".py .js"</code></td></tr>
<tr><td><code>--ignore</code></td><td>str</td><td>Ignored paths</td><td><code>--ignore "node_modules"</code></td></tr>
<tr><td><code>--mode</code></td><td>enum</td><td>none / default / shallow / deep</td><td><code>--mode deep</code></td></tr>
<tr><td><code>--format</code></td><td>enum</td><td>markdown / xml / plain / jsonl_chunk</td><td><code>--format xml</code></td></tr>
<tr><td><code>--minify</code></td><td>flag</td><td>Enable minification</td><td><code>--minify</code></td></tr>
<tr><td><code>--no-comments</code></td><td>flag</td><td>Strip comments</td><td><code>--no-comments</code></td></tr>
<tr><td><code>--no-secrets</code></td><td>flag</td><td>Mask secrets</td><td><code>--no-secrets</code></td></tr>
<tr><td><code>--skeleton</code></td><td>flag</td><td>Skeleton mode</td><td><code>--skeleton</code></td></tr>
<tr><td><code>--output</code></td><td>str</td><td>Output file</td><td><code>--output out.txt</code></td></tr>
<tr><td><code>--stdout</code></td><td>flag</td><td>Print to stdout</td><td><code>--stdout</code></td></tr>
<tr><td><code>--git</code></td><td>flag</td><td>Git changes only</td><td><code>--git</code></td></tr>
<tr><td><code>--gitignore</code></td><td>flag</td><td>Respect .gitignore</td><td><code>--gitignore</code></td></tr>
<tr><td><code>--tree</code></td><td>flag</td><td>File tree</td><td><code>--tree</code></td></tr>
<tr><td><code>--mermaid</code></td><td>flag</td><td>Mermaid graph</td><td><code>--mermaid</code></td></tr>
<tr><td><code>--dependencies</code></td><td>flag</td><td>Dependency map</td><td><code>--dependencies</code></td></tr>
<tr><td><code>--patch</code></td><td>str</td><td>LLM JSON patch</td><td><code>--patch patch.json</code></td></tr>
<tr><td><code>--template</code></td><td>str</td><td>Jinja2 template</td><td><code>--template my.j2</code></td></tr>
<tr><td><code>--system-prompt</code></td><td>str</td><td>Custom system prompt</td><td><code>--system-prompt "Review"</code></td></tr>
</tbody>
</table>

<h3>Examples</h3>
<pre># Minimal run
python main.py --cli --path ./myapp --stdout

# Full analysis with XML
python main.py --cli --path ./myapp --ext ".py .js .ts" --ignore "node_modules,.git,__pycache__" --mode deep --mermaid --tree --dependencies --minify --no-comments --skeleton --format xml --output analysis.xml

# Git diff
python main.py --cli --path ./myapp --git --gitignore --stdout

# LLM JSON patch
python main.py --cli --path ./myapp --patch llm_response.json

# Custom Jinja2 template
python main.py --cli --path ./myapp --template my.j2 --stdout

# Mermaid diagram
python main.py --cli --path ./myapp --mode deep --mermaid --output with_mermaid.md

# Multiple paths
python main.py --cli --path ./frontend ./backend --format xml --output combined.xml</pre>

<hr>

<h2>🏗️ Tech Stack</h2>
<table>
<thead><tr><th>Component</th><th>Technology</th></tr></thead>
<tbody>
<tr><td>Language</td><td>Python 3.10+</td></tr>
<tr><td>GUI Framework</td><td>PySide6 (Qt 6)</td></tr>
<tr><td>Architecture</td><td>Clean Architecture + Redux-like</td></tr>
<tr><td>Tokenization</td><td>tiktoken (OpenAI)</td></tr>
<tr><td>Templating</td><td>jinja2 (11 built-in)</td></tr>
<tr><td>AST parsers</td><td>ast (Python), tree-sitter (JS/TS/Go/Rust)</td></tr>
<tr><td>Distribution</td><td>PyInstaller, AUR</td></tr>
</tbody>
</table>

<hr>

<h2>📁 Project Structure</h2>
<pre>CodeContext/
├── main.py                  # Entry point
├── VERSION.txt              # Version
├── requirements.txt         # Dependencies
├── assets/images/logo.png   # Logo
├── aur_build/               # AUR package
│   ├── PKGBUILD
│   ├── codecontext.desktop
│   └── codecontext.sh
├── themes/                  # JSON themes
└── src/
    ├── main_app.py          # Bootstrap
    ├── store/               # Redux-like (state.py, store.py)
    ├── controllers/         # Business logic
    ├── ui/                  # PySide6
    │   ├── main_window.py
    │   ├── dialogs.py
    │   ├── theme_manager.py
    │   └── components/
    │       ├── sidebar.py, action_panel.py, folder_list.py
    │       ├── file_tree.py, log_panel.py, status_bar.py
    │       ├── empty_state.py, analytics_panel.py
    └── utils/
        ├── config.py, async_runtime.py</pre>

<hr>

<h2>🗺️ Roadmap</h2>
<ul>
<li>🍎 macOS Finder context menu</li>
<li>🤖 Direct OpenAI/Anthropic API integration</li>
<li>🏛️ Hexagonal Architecture analysis</li>
<li>🔌 Plugin system</li>
<li>🌐 In-app i18n</li>
</ul>

<hr>

<h2>👨‍💻 Team</h2>
<p><b>Developer:</b> mcniki · <a href="https://vk.com/gor_niki">VK: gor_niki</a> · Issues & PRs on GitHub</p>

<hr>

<h2>🤝 Contributing</h2>
<ol>
<li>Fork the repository</li>
<li>Branch: <code>git checkout -b feature/AmazingFeature</code></li>
<li>Commit: <code>git commit -m 'Add AmazingFeature'</code></li>
<li>Push: <code>git push origin feature/AmazingFeature</code></li>
<li>Pull Request</li>
</ol>
<p>Follow SOLID principles (see <code>docs/ARCHITECTURE.md</code>).</p>

<hr>

<h2>📄 License</h2>
<p>MIT. See <code>LICENSE</code> for details.</p>

</div>

<style>
  .lang-btn {
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
  .lang-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 3px 10px rgba(0,0,0,0.12);
    border-color: #0969da;
    color: #0969da;
  }
  .lang-ru, .lang-en { display: none; }

  /* Default: show Russian */
  #lang_ru:checked ~ .lang-header .lang-btn[for="lang_ru"],
  #lang_en:checked ~ .lang-header .lang-btn[for="lang_en"] {
    background: #0969da;
    border-color: #0969da;
    color: #fff;
    box-shadow: 0 3px 10px rgba(9,105,218,0.35);
  }

  #lang_ru:checked ~ .lang-ru { display: block; }
  #lang_en:checked ~ .lang-en { display: block; }
</style>
