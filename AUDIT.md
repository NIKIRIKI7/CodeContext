# Ponytail Audit — подробно

**Дата:** 2026-06-16
**Правило:** Остановись на первой ступеньке лестницы, которая держит. Удаление перед добавлением. Stdlib раньше кода. Никаких спекулятивных абстракций.

---

## Ранжирование (самые жирные cuts первые)

---

### 1. `delete` `tests/test_monorepo_support.py` — 211 строк мёртвых тестов

**Файл:** `tests/test_monorepo_support.py`

**Проблема:** Файл импортирует `MonorepoImportStrategy` из несуществующего модуля:

```python
from src.services.strategies.import_strategies import MonorepoImportStrategy
from src.services.import_resolution_service import ImportResolutionService
```

Модуль `src/services/strategies/import_strategies.py` удалён/переименован. Все 26 тестов (26 `assert`) падают с `ModuleNotFoundError`. `ImportResolutionService` существует, но без стратегии бесполезен.

**Решение:** Удалить весь файл. Если монорепо-поддержка понадобится — тесты писать заново с актуальным API.

---

### 2. `delete` `tests/test_ci_cd_integration.py` — 115 строк битых тестов

**Файл:** `tests/test_ci_cd_integration.py`

**Проблема:** Импортирует `Store` из несуществующего `src.store.store`:

```python
from src.store.store import Store       # ← ModuleNotFoundError
```

Использует устаревший конструктор `ScanWorkspaceUseCase`:

```python
uc = ScanWorkspaceUseCase(mock_dispatcher, mock_file_service, mock_fs_repo)
# Реальный конструктор: ScanWorkspaceUseCase(state, file_service, fs_repo)
```

Все 4 теста (`@pytest.mark.asyncio`) не выполняются.

**Решение:** Удалить весь файл.

---

### 3. `stdlib` `integration_strategies.py:_create_workflow` — 250+ строк hand-rolled XML

**Файл:** `src/services/strategies/integration_strategies.py:380-647`

**Проблема:** Ручная генерация двух XML-файлов (`Info.plist` и `document.wflow`) через f-строки с ручным XML-escaping:

```python
info_plist = f'''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>{bundle_id}</string>
    ...'''
```

И ещё 200 строк для `document.wflow`. Вручную экранирует `&`, `<`, `>`:

```python
escaped = command.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
```

**Решение:** Использовать `plistlib` (stdlib, всегда в Python):

```python
import plistlib

def _create_workflow(self, name: str, command: str) -> None:
    services_dir = os.path.expanduser("~/Library/Services")
    os.makedirs(services_dir, exist_ok=True)
    workflow_path = os.path.join(services_dir, f"{name}.workflow")
    if os.path.exists(workflow_path):
        shutil.rmtree(workflow_path)

    contents_dir = os.path.join(workflow_path, "Contents")
    os.makedirs(contents_dir, exist_ok=True)

    # Info.plist
    info = {
        "CFBundleIdentifier": f"com.codecontext.ai.{uuid.uuid4().hex}",
        "CFBundleName": name,
        "CFBundlePackageType": "BNDL",
        "CFBundleShortVersionString": "1.0",
        "CFBundleSignature": "????",
        "CFBundleVersion": "1.0",
        "NSServices": [{
            "NSMenuItem": {"default": name},
            "NSMessage": "runWorkflowAsService",
            "NSSendFileTypes": ["public.folder", "public.directory"],
        }],
    }
    with open(os.path.join(contents_dir, "Info.plist"), "wb") as f:
        plistlib.dump(info, f)

    # document.wflow — та же схема с plistlib.dump()
```

Сокращение: с ~270 строк до ~20-30.

---

### 4. `shrink` `pyproject.toml` — дублирующийся entry point

**Файл:** `pyproject.toml:45-46,62-63`

```toml
[project.scripts]
codecontext = "codecontext:main"        # строка 46

[project.entry-points.gui_scripts]
codecontext = "codecontext:main"        # строка 63 — идентично!
```

`gui_scripts` на Windows создаёт `.exe`-обёртку без консоли. Если это различие не нужно — лишняя секция. Если нужно — оставить только `gui_scripts`, убрать `scripts`.

**Решение:** Удалить `[project.entry-points.gui_scripts]` (или `[project.scripts]`, если консоль не нужна).

---

### 5. `yagni` `scan_use_case.py` — кэш токенов, который никогда не попадает

**Файл:** `src/use_cases/scan_use_case.py:18-34`

```python
token_cache = self._load_cache()
new_cache = {}
for path in file_paths:
    stat = os.stat(path)
    cache_key = f"{path}_{stat.st_size}_{stat.st_mtime}"
    if cache_key in token_cache:          # ← никогда не совпадёт
        tokens = token_cache[cache_key]
    else:
        tokens = stat.st_size // 4
    new_cache[cache_key] = tokens

self._save_cache(new_cache)
```

Ключ кэша включает `st_mtime` (наносекунды). Файл перечитывается сразу после сканирования — `st_mtime` гарантированно изменится. Кэш всегда промахивается. Пишет JSON-файл на диск каждый раз.

**Решение:** Удалить `_load_cache`, `_save_cache` и всю кэш-логику (строки 18-34, 67, 93). `tokens = stat.st_size // 4` вычисляется мгновенно, IO здесь не узкое место.

---

### 6. `delete` `pipeline_utils.py` — мёртвый последовательный метод

**Файл:** `src/utils/pipeline_utils.py:60-70`

```python
@staticmethod
def process_files_batch(raw_files, options):
    # serial version — никем не вызывается
    ...

@staticmethod
def process_files_batch_parallel(raw_files, options):
    # единственный используемый
    ...
```

Метод `process_files_batch` — полный дубликат логики `process_files_batch_parallel`, только без `ThreadPoolExecutor`. Ни одного вызова во всей кодовой базе.

**Решение:** Удалить метод `process_files_batch` (строки 60-70).

---

### 7. `shrink` `patch_use_case.py` — 3 запасных regex, из которых нужен только первый

**Файл:** `src/use_cases/patch_use_case.py:13-33`

```python
# Fallback 1 — покрывает >99% случаев
blocks = re.findall(r'```(?:json)?\s*(.*?)\s*```', patch_str, re.DOTALL)

if not blocks:                          # Fallback 2 — ловит голый JSON-массив
    match = re.search(r'\[\s*\{.*?\}\s*\]', patch_str, re.DOTALL)
    ...

if not blocks:                          # Fallback 3 — ловит голый JSON-объект
    match = re.search(r'\{\s*"action".*?\}', patch_str, re.DOTALL)
    ...
```

Fallback 2 и 3 никогда не срабатывают: LLM (все известные) всегда оборачивают JSON в ```json ... ```. Если понадобится — добавить в момент, когда тест покажет обратное.

**Решение:** Удалить fallback 2 и 3 (строки 22-31).

---

### 8. `yagni` `di_container.py:37` — избыточный SimpleNamespace

**Файл:** `src/di_container.py:37`

```python
self.ui_registry = SimpleNamespace(sidebar_tabs={}, action_buttons={})
```

Строкой ниже:

```python
self.plugin_api = PluginAPI("core", self.state, self, self.ui_registry)
```

А `PluginAPI.__init__` уже имеет:

```python
if ui_registry is None:
    ui_registry = SimpleNamespace(sidebar_tabs={}, action_buttons={})
self.ui = ui_registry
```

То есть дефолт есть в обоих местах. Повтор.

**Решение:** Убрать `self.ui_registry` из DIContainer, передавать `None` в PluginAPI и PluginManager, или завести один раз.

---

### 9. `yagni` `plugin_api.py:16-19` — hasattr-защита при гарантированном наличии

**Файл:** `src/api/plugin_api.py:16-19`

```python
if not hasattr(self.ui, 'sidebar_tabs'):
    self.ui.sidebar_tabs = {}
if not hasattr(self.ui, 'action_buttons'):
    self.ui.action_buttons = {}
```

`self.ui` это либо `SimpleNamespace(sidebar_tabs={}, action_buttons={})` (дефолт), либо объект от `DIContainer`, у которого эти атрибуты тоже есть. `hasattr` никогда не вернёт `False`.

**Решение:** Удалить guard-блок (строки 16-19).

---

### 10. `yagni` `tour_service.py` — класс с одним методом

**Файл:** `src/services/tour_service.py:5-57`

```python
class TourService:
    """Доменный сервис интерактивного тура."""

    def get_tour_steps(self) -> List[Dict[str, str]]:
        return [
            {"title": tr("tour_service.welcome.title"), "text": tr("tour_service.welcome.text")},
            ...
        ]
```

Ни состояния, ни инъекции, ни наследования. Просто список диктов.

**Решение:** Заменить на константу:

```python
TOUR_STEPS: List[Dict[str, str]] = [
    {"title": tr("tour_service.welcome.title"), "text": tr("tour_service.welcome.text")},
    ...
]
```

Убрать класс. `main_controller.py` будет импортировать `TOUR_STEPS` напрямую.

---

### 11. `yagni` `settings_repository.py` — класс-обёртка для `json.load`/`json.dump`

**Файл:** `src/data/settings_repository.py:1-29`

```python
class SettingsRepository:
    def __init__(self, filename: str = "user_settings.json"):
        self.filepath = os.path.join(get_app_data_dir(), filename)

    def load(self) -> Dict[str, Any]:
        if not os.path.exists(self.filepath):
            return {}
        with open(self.filepath, 'r') as f:
            return json.load(f)

    def save(self, settings_dict):
        with open(self.filepath, 'w') as f:
            json.dump(settings_dict, f, ensure_ascii=False, indent=2)
```

7 строк логики, обёрнутых в класс с конструктором.

**Решение:** Две свободные функции:

```python
def load_settings(path=None):
    path = path or os.path.join(get_app_data_dir(), "user_settings.json")
    if not os.path.exists(path):
        return {}
    with open(path, 'r') as f:
        return json.load(f)

def save_settings(settings_dict, path=None):
    path = path or os.path.join(get_app_data_dir(), "user_settings.json")
    with open(path, 'w') as f:
        json.dump(settings_dict, f, ensure_ascii=False, indent=2)
```

---

### 12. `shrink` `formatting_service.py` — `maxsize=5` для одного шаблона

**Файл:** `src/services/formatting_service.py:106-108`

```python
@lru_cache(maxsize=5)
def _get_jinja_env(template_dir: str):
    return Environment(loader=FileSystemLoader(template_dir), ...)
```

Пул на 5 окружений Jinja, хотя в рантайме загружается ровно один шаблон (кастомный). Разные `template_dir` не используются.

**Решение:** `@lru_cache(maxsize=1)` или `@lru_cache` без аргумента (дефолт 128 — всё ещё перебор, но не принципиально).

---

### 13. `shrink` `github_service.py` — `functools.partial` для одного вызова

**Файл:** `src/services/github_service.py:7,55`

```python
import functools

# ...
await asyncio.to_thread(functools.partial(shutil.rmtree, dest_path, ignore_errors=True))
```

`functools.partial` вызван ровно один раз, для оборачивания `shutil.rmtree` с одним предзаполненным аргументом.

**Решение:** `lambda` или прямой вызов:

```python
await asyncio.to_thread(lambda: shutil.rmtree(dest_path, ignore_errors=True))
```

И удалить `import functools`.

---

### 14. `delete` `.gitignore:54-57` — избыточные пути

```
__pycache__/              # ← строка 2, уже покрывает всё
...
src/__pycache__/          # строка 54 — дубль
src/logic/__pycache__/    # строка 55 — дубль
src/services/__pycache__/ # строка 56 — дубль
src/ui/__pycache__/       # строка 57 — дубль
```

Правило `__pycache__/` на строке 2 уже игнорирует все `__pycache__` на любом уровне вложенности.

**Решение:** Удалить строки 54-57.

---

### 15. `delete` `pipeline_utils.py:2` — `import math` не используется

```python
import math      # ← never referenced in file
```

**Решение:** Удалить строку.

---

### 16. `delete` `file_service.py:4` — `import time` не используется

```python
import time      # ← never referenced in file
```

Ни одного `time.sleep()` или `time.time()` в файле.

**Решение:** Удалить строку.

---

### 17. `delete` `theme_manager.py:485` — мёртвый аргумент format()

```python
qss = QSS_TEMPLATE.format(
    ...
    c_secondary_fg=selected_mode_colors.get("secondary_fg", "#000000"),  # строка 485
    ...
)
```

`{c_secondary_fg}` ни разу не встречается в `QSS_TEMPLATE`. Параметр передаётся в `format()` но никуда не подставляется.

**Решение:** Удалить строку 485.

---

### 18. `delete` `analytics_panel.py:3` — `QHBoxLayout` не используется

```python
from PySide6.QtWidgets import QWidget, QVBoxLayout, QTableWidget, QTableWidgetItem, \
    QHeaderView, QProgressBar, QHBoxLayout
```

`QHBoxLayout` нигде не вызывается в файле. Панель использует только `QVBoxLayout`.

**Решение:** Удалить `QHBoxLayout` из импорта.

---

### 19. `delete` `sidebar.py:2` — `import sys` не используется

```python
import sys        # ← never referenced in file
```

**Решение:** Удалить строку.

---

### 20. `delete` `main_window.py:5` — `Signal`, `QObject` не используются

```python
from PySide6.QtCore import Signal, QObject, Qt, QPropertyAnimation, QTimer
```

`Signal` и `QObject` импортированы, но в файле `main_window.py` ни разу не упоминаются (только `Qt`, `QPropertyAnimation`, `QTimer` используются). `Signal` и `QObject` остались от рефакторинга.

**Решение:** Удалить `Signal, QObject` из импорта.

---

### 21. `shrink` `cli_controller.py` — `time.sleep(3)` вместо `input()`

**Файл:** `src/controllers/cli_controller.py:208-211`

```python
@staticmethod
def _keep_window_open():
    print(tr("cli_controller.window_closing"))
    time.sleep(3)
```

Ждёт 3 секунды вслепую. Пользователь всё равно не может прочитать сообщение, если закрывается.

**Решение:** Заблокировать до Enter:

```python
@staticmethod
def _keep_window_open():
    input(tr("cli_controller.window_closing") + " ")
```

Ponytail: если stdin не TTY — пропустить. Можно добавить `sys.stdin.isatty()` guard.

---

### 22. `shrink` `output_service.py` — Qt-запасной путь, который никогда не используется

**Файл:** `src/services/output_service.py:5,8-18`

```python
from PySide6.QtGui import QGuiApplication

def copy_to_clipboard(text: str):
    # ponytail: Qt-буфер теряет данные при мгновенном выходе из CLI.
    # Возвращаем надежный pyperclip для работы без event loop.
    try:
        import pyperclip
        pyperclip.copy(text)
        return
    except ImportError:
        app = QGuiApplication.instance() or QGuiApplication(sys.argv)
        app.clipboard().setText(text)
        app.processEvents()
```

В комментарии `ponytail:` сказано, что Qt-путь ненадёжен. `pyperclip` есть в зависимостях `pyproject.toml`. Если нет — либо установить, либо упасть с понятной ошибкой.

**Решение:** Убрать Qt fallback:

```python
def copy_to_clipboard(text: str):
    import pyperclip
    pyperclip.copy(text)
```

И удалить `from PySide6.QtGui import QGuiApplication`.

---

### 23. `shrink` `file_tree.py` — два идентичных рекурсивных метода

**Файл:** `src/ui/components/file_tree.py:163-185`

```python
def _propagate_check(self, parent_item, state):         # строка 163
    self._is_updating = True
    for row in range(parent_item.rowCount()):
        child = parent_item.child(row)
        child.setCheckState(Qt.Checked if state else Qt.Unchecked)
        path = child.data(Qt.UserRole)
        is_file = child.data(Qt.UserRole + 1)
        if path and is_file:
            self.on_toggle(path, state)
        if child.hasChildren():
            self._propagate_check_recursive(child, state)
    self._is_updating = False

def _propagate_check_recursive(self, parent_item, state):  # строка 176
    for row in range(parent_item.rowCount()):
        child = parent_item.child(row)
        child.setCheckState(Qt.Checked if state else Qt.Unchecked)
        path = child.data(Qt.UserRole)
        is_file = child.data(Qt.UserRole + 1)
        if path and is_file:
            self.on_toggle(path, state)
        if child.hasChildren():
            self._propagate_check_recursive(child, state)
```

Разница: в первом есть `_is_updating` guard, во втором — нет. Тело цикла идентично.

**Решение:** Объединить: `_propagate_check` принимает необязательный флаг `set_updating=True`, вызывает себя рекурсивно. Убрать `_propagate_check_recursive`.

---

### 24. `yagni` `main_controller.py` — `tour_service` мог бы быть функцией

**Файл:** `src/controllers/main_controller.py:33,47,300`

```python
self._tour_service = tour_service

# Единственное использование:
self.state.tour_steps = self._tour_service.get_tour_steps()
```

`TourService` — класс с одним методом без состояния (см. finding #10). Впрыскивается через конструктор, но мог бы быть импортирован как константа.

**Решение:** Импортировать `TOUR_STEPS` напрямую, удалить `tour_service` из конструктора `MainController`.

---

### 25. `shrink` `di_container.py` — `@property` создаёт новые инстансы при каждом доступе

**Файл:** `src/di_container.py:42-92`

```python
@property
def file_service(self):
    return FileService(self.fs_repo)          # новый объект каждый раз

@property
def github_service(self):
    return GitHubService()                    # новый объект каждый раз
# и так 8 раз
```

Каждый доступ через `container.file_service` создаёт новый экземпляр. Без сайд-эффектов (нет состояния), но лишняя аллокация.

**Решение:** `@functools.cached_property` для stateless сервисов (все, кроме `main_controller` и `cli_controller`, которые уже кэшируются через `hasattr`).

```python
@functools.cached_property
def file_service(self):
    return FileService(self.fs_repo)
```

---

### 26. `stdlib` `pipeline_utils.py` — hand-rolled BFS вместо asyncio.TaskGroup

**Файл:** `src/utils/pipeline_utils.py:73-91`

```python
@staticmethod
async def resolve_and_collect_dependencies_async(initial_queue, visited_paths, all_paths, is_deep, fs_repo):
    queue = initial_queue
    while queue:
        curr_path, depth = queue.pop(0)   # O(n) pop
        if curr_path in visited_paths:
            continue
        ...
        for imp in imports:
            for p in import_resolution_service.resolve(imp, all_paths):
                if p not in visited_paths:
                    queue.append((p, depth + 1))
```

Hand-rolled BFS с `list.pop(0)` — O(n²). Python 3.11+ имеет `asyncio.TaskGroup`.

**Решение:** Использовать `asyncio.Queue` + `TaskGroup` для конкурентного разрешения зависимостей. Но это не срочно — метод вызывается редко. Ponytail: оставить как есть, пометить `# ponytail: list.pop(0), заменить на collections.deque если очередь станет >1000`.

---

### 27. `delete` `src/services/` — отсутствует `__init__.py`

`src/api/__init__.py` — есть
`src/data/__init__.py` — есть
`src/i18n/__init__.py` — есть
`src/store/__init__.py` — есть
`src/ui/__init__.py` — есть
`src/utils/__init__.py` — есть
**`src/services/__init__.py` — нет**

Python 3.3+ работает с implicit namespace packages, поэтому код не падает. Но неконсистентно с остальными пакетами, и некоторые тулы (mypy, coverage) могут сбиваться.

**Решение:** Добавить пустой `src/services/__init__.py`.

---

## Итог

```
net: -623 lines, 0 deps possible.
```

| Категория | Строк | Штук |
|-----------|-------|------|
| Удаление битых тестов | -326 | 2 |
| plistlib вместо hand-rolled XML | -250 | 1 |
| Мёртвые импорты и аргументы | -27 | 6 |
| YAGNI-классы/защиты | -20 | 5 |
| Дублирующийся entry point | -3 | 1 |
| Избыточный .gitignore | -4 | 1 |
| Отсутствующий __init__.py | 0 | 1 |

0 новых зависимостей — всё решается stdlib (`plistlib`) или удалением.

`Lean already. Ship.` когда эти 27 finding'ов адресованы.
