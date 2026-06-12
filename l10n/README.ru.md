<div align="center">

[🇬🇧 English](../README.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇨🇳 中文](README.zh.md) · [🇪🇸 Español](README.es.md) · [🇮🇹 Italiano](README.it.md) · [🇸🇦 العربية](README.ar.md) · [🇧🇷 Português](README.pt.md) · [🇯🇵 日本語](README.ja.md) · [🇰🇷 한국어](README.ko.md) · [🇮🇳 हिन्दी](README.hi.md) · [🇹🇷 Türkçe](README.tr.md) · [🇳🇱 Nederlands](README.nl.md) · [🇵🇱 Polski](README.pl.md)

<br>

# CodeContext AI

<img src="https://raw.githubusercontent.com/NIKIRIKI7/CodeContext/main/assets/images/logo.png" alt="CodeContext AI Logo" width="120"/>

**AI-powered codebase analysis & prompt preparation tool**

[![AUR](https://img.shields.io/aur/version/codecontext-ai?style=flat-square&logo=archlinux&label=AUR)](https://aur.archlinux.org/packages/codecontext-ai)
[![Version](https://img.shields.io/badge/version-1.25.1-blue?style=flat-square)](../VERSION.txt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue?style=flat-square&logo=python)](https://www.python.org/downloads/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey?style=flat-square)]()

</div>

<h2>🌟 О проекте</h2>

<p><b>CodeContext AI</b> — мощный десктопный инструмент для подготовки кодовой базы к работе с большими языковыми моделями (LLM). Сканирует папки проекта, анализирует структуру, строит граф зависимостей и генерирует единый структурированный промпт.</p>

<h3>❓ Зачем?</h3>
<p>При работе с ИИ разработчики упираются в лимит токенов — нейросеть «теряет» архитектуру проекта, когда код копируется частями. <b>CodeContext AI решает это</b>: в пару кликов собирает ВЕСЬ проект в один промпт, экономя до 80% токенов.</p>

<hr>

<h2>🚀 Возможности</h2>

<table>
<thead><tr><th>Возможность</th><th>CodeContext AI</th><th>Вручную</th></tr></thead>
<tbody>
<tr><td>🗜️ Minify</td><td><b>До 80%</b> экономии токенов — удаляет пробелы и пустые строки</td><td>Копировать вручную</td></tr>
<tr><td>🧩 LLM Patcher</td><td>Предпросмотр и JSON-патчи</td><td>Нет</td></tr>
<tr><td>✅ LLM Checker</td><td>Авто-проверка кода перед записью</td><td>Нет</td></tr>
<tr><td>🔗 AST-граф</td><td>Python, JS/TS, Vue</td><td>Только файлы</td></tr>
<tr><td>🖱️ Контекстное меню</td><td>Windows / Linux</td><td>Нет</td></tr>
<tr><td>🎨 Темы</td><td>Apple, Modern, кастомные</td><td>Фиксированный UI</td></tr>
<tr><td>⚙️ Кастомизация (v1.14+)</td><td>Premiere Pro-style</td><td>Фиксированный UI</td></tr>
<tr><td>🌐 i18n (v1.17+)</td><td>15 языков, автоопределение системы</td><td>Один язык</td></tr>
<tr><td>♻️ Дедупликация (v1.23+)</td><td>Находит и исключает файлы с одинаковым содержимым</td><td>Вручную</td></tr>
<tr><td>⚡ Агрессивная миниф. (v1.23+)</td><td>Доп. сжатие — удаляет хвостовые пробелы в каждой строке</td><td>Вручную</td></tr>
<tr><td>📌 Контр. точки (v1.23+)</td><td>Сохранение снимков ДО/ПОСЛЕ для отладки</td><td>Нет</td></tr>
<tr><td>👁️ Авто-слежка (v1.23+)</td><td>Следит за файлами и перезапускает обработку при изменениях</td><td>Нет</td></tr>
<tr><td>🔌 Плагины (v1.25+)</td><td>Расширение Python-плагинами — кастомные вкладки, кнопки, i18n</td><td>Нет</td></tr>
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

<h3>PyPI (pip)</h3>
<pre>pip install codecontext-ai</pre>

<pre># Запуск:
codecontext</pre>

<h3>Windows .exe</h3>
<pre>pip install pyinstaller
pyinstaller --windowed --onefile --icon=assets/images/logo.ico --name "CodeContext AI" main.py</pre>

<h3>Arch Linux (AUR)</h3>
<table>
<thead><tr><th>Действие</th><th>Команда</th></tr></thead>
<tbody>
<tr><td>Установка</td><td><code>yay -S codecontext-ai</code></td></tr>
<tr><td>Поиск</td><td><code>yay -Ss codecontext</code></td></tr>
<tr><td>Обновление</td><td><code>yay -Syu</code></td></tr>
<tr><td>Удаление</td><td><code>sudo pacman -Rns codecontext-ai</code></td></tr>
</tbody>
</table>
<p>Если <b>yay</b> не установлен:</p>
<pre>sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si</pre>
<p>Альтернативы: <code>paru -S codecontext-ai</code></p>

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
<tr><td>☑ Minify</td><td>Обрезает пробелы в начале/конце каждой строки, удаляет пустые строки — безопасное сжатие для повседневного использования</td></tr>
<tr><td>☑ Агрессивная</td><td>Дополнительный проход минификации — агрессивно удаляет хвостовые пробелы в каждой строке. Включайте вместе с Minify для макс. экономии токенов, когда контекст ограничен</td></tr>
<tr><td>☑ No Comments</td><td>Вырезает все комментарии из кода</td></tr>
<tr><td>☑ No Secrets</td><td>Маскирует потенциальные секреты (ключи API, пароли, токены)</td></tr>
<tr><td>☑ Skeleton ☠️</td><td><b>Удаляет тела функций</b>, оставляя только названия и структуру классов — макс. экономия токенов</td></tr>
<tr><td>☑ Дедупликация</td><td>Сканирует все файлы и исключает дубликаты с идентичным содержимым — устраняет повторяющийся контекст из скопированных файлов</td></tr>
<tr><td>☑ Контр. точки</td><td>Сохраняет промежуточные снимки обработки (ДО/ПОСЛЕ) на диск — полезно для отладки этапов пайплайна или сравнения результатов</td></tr>
<tr><td>☑ Авто-слежка</td><td>Отслеживает изменения в файлах проекта и автоматически перезапускает обработку — держит промпт актуальным во время активной разработки</td></tr>
<tr><td>Формат</td><td>Markdown, XML, Plain, JSONL Chunks, Custom (Jinja2)</td></tr>
<tr><td>📁 шаблон</td><td>Выбор Jinja2-шаблона (активно при формате Custom)</td></tr>
</tbody>
</table>

<p><b>Skeleton Mode:</b> удаляет реализацию функций (<code>def func_name(...):  # ... implementation ...</code>), оставляя все классы и методы — LLM видит архитектуру гигантского проекта, тратя минимум токенов.</p>

<p><b>Minify vs Агрессивная:</b> <b>Minify</b> обрезает пробелы в начале/конце строк и удаляет пустые строки — безопасно для любой кодовой базы, снижает токены без потери читаемости. <b>Агрессивная</b> добавляет дополнительный проход, устраняющий хвостовые пробелы в каждой строке для максимального сжатия. Включайте обе опции, когда нужно уместить больше кода в ограниченный контекст LLM.</p>

<p><b>Дедупликация:</b> автоматически находит файлы с идентичным содержимым по всему проекту и исключает дубликаты из вывода — предотвращает попадание одного и того же кода в промпт дважды и лишнюю трату токенов.</p>

<p><b>Контрольные точки:</b> сохраняет промежуточные результаты на каждом этапе пайплайна (до очистки, после минификации и т.д.) в папку <code>checkpoints/</code>. Полезно для отладки каждого шага обработки или сравнения результатов рядом.</p>

<p><b>Авто-слежка:</b> отслеживает изменения в файлах проекта через системный файловый наблюдатель. При сохранении файла пайплайн автоматически запускается заново — идеально во время активной разработки, когда нужны непрерывные обновления промпта.</p>

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

<h3>12. 🔌 Система плагинов (v1.25+)</h3>
<p><b>CodeContext AI</b> поддерживает <b>систему Python-плагинов</b>, позволяющую расширять приложение кастомным функционалом.</p>

<h4>📁 Структура плагина</h4>
<pre>my_plugin/
├── manifest.json          # Метаданные плагина
├── requirements.txt       # (Опционально) pip-зависимости
├── locales/
│   ├── en.json            # Английские переводы
│   └── ru.json            # Русские переводы
└── plugin.py              # Точка входа</pre>

<h4>📄 manifest.json</h4>
<pre>{
  "id": "my_plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Делает что-то полезное",
  "entry_point": "plugin"
}</pre>

<h4>🐍 plugin.py (Пример)</h4>
<pre>from src.api.plugin_api import IPlugin, PluginAPI

class MyPlugin(IPlugin):
    id = "my_plugin"
    name = "My Plugin"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        # Переводы из папки locales/ загружаются автоматически
        # Регистрируем вкладку в боковой панели
        api.ui.register_sidebar_tab(
            "my_tab", "My Tab",
            lambda: QLabel("Привет от плагина!")
        )
        # Регистрируем кнопку действия
        api.ui.register_action_button(
            "my_action", "My Action",
            lambda: api.add_log("Клик по плагину")
        )
        api.add_log("Мой плагин инициализирован")

    def on_shutdown(self) -> None:
        pass</pre>

<h4>🔐 Безопасность</h4>
<ul>
<li>Плагины получают <b>полный доступ к Python</b> — устанавливайте только из доверенных источников</li>
<li>При первой загрузке диалог безопасности запрашивает ваше разрешение</li>
<li>Если есть <code>requirements.txt</code>, вы увидите лог pip install перед загрузкой</li>
<li>Одобренные плагины запоминаются в настройках (<code>approved_plugins</code>)</li>
</ul>

<h4>🛠 Plugin API</h4>
<table>
<thead><tr><th>Свойство / Метод</th><th>Описание</th></tr></thead>
<tbody>
<tr><td><code>api.store</code></td><td>Redux-стор (только чтение, доступ: <code>state.settings.xxx</code>)</td></tr>
<tr><td><code>api.dispatcher</code></td><td>Отправка действий (напр. <code>UI_ADD_LOG</code>)</td></tr>
<tr><td><code>api.ui.register_sidebar_tab(id, label, factory)</code></td><td>Добавить вкладку в боковую панель</td></tr>
<tr><td><code>api.ui.register_action_button(id, label, callback)</code></td><td>Добавить кнопку в меню «Плагины 🔽»</td></tr>
<tr><td><code>api.add_translations(lang, data)</code></td><td>Добавить переводы (сливаются поверх встроенных)</td></tr>
<tr><td><code>api.add_log(message)</code></td><td>Записать в панель логов</td></tr>
</tbody>
</table>

<h4>⚙️ Видимость</h4>
<p>Вкладки и кнопки плагинов можно включать/выключать через <b>⚙ Кастомизацию интерфейса</b> — они появляются рядом со встроенными со своими чекбоксами.</p>

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

<h2>🗺️ Дорожная карта</h2>
<ul>
<li>📚 <b>RAG (Retrieval-Augmented Generation) режим</b> — индексация огромных кодовых баз в векторную БД (Chroma/FAISS) для контекстного поиска.</li>
<li>🚫 <b>Глубокий парсинг .gitignore</b> — поддержка вложенных файлов <code>.gitignore</code> и глобального <code>~/.gitignore</code>.</li>
<li>☁️ <b>Облачная синхронизация</b> — синхронизация пресетов, настроек и промптов через GitHub Gists.</li>
<li>🌳 <b>Мульти-root Workspaces</b> — продвинутая поддержка монорепозиториев (Lerna, NX, Turborepo).</li>
<li>🚀 <b>Интеграция с пайплайнами CI/CD</b> — плагины для GitHub Actions и GitLab CI для генерации контекста в PR.</li>
<li>🤖 <b>Прямой коннект с OpenAI/Anthropic API</b> — замыкание полного цикла от генерации промпта до сохранения итогового кода.</li>
<li>🍎 macOS Finder context menu</li>
<li>🔌 Плагинная система ✅</li>
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
