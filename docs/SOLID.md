# SOLID — Принципы и найденные нарушения

## Обзор найденных нарушений

Ниже — полный разбор нарушений SOLID в исходном коде и способы их устранения.

---

## S — Single Responsibility Principle

**Правило:** Каждый класс/модуль должен иметь одну причину для изменения.

### Нарушение 1: `MainController` — «Бог-объект»

```python
# ❌ ДО: MainController знает про всё
class MainController:
    def __init__(self, store, dispatcher):
        self.fs_repo = FileSystemRepository()        # создаёт зависимости сам
        self.settings_repo = SettingsRepository()    # (нарушение DIP)
        self.file_service = FileService(...)
        self.process_service = ProcessingService(...)
        self.cleaner_service = CleanerService()
        # ... ещё 10 сервисов ...

    def load_initial_settings(self): ...    # работа с настройками
    def save_workspace(self): ...           # работа с воркспейсом
    def add_github_repo(self): ...          # GitHub
    def scan_only(self): ...                # сканирование
    def start_processing(self): ...         # обработка
    def _format_output(self): ...           # форматирование
    # итого: 20+ методов, 6+ зон ответственности
```

```python
# ✅ ПОСЛЕ: Контроллер только координирует Use Cases
class MainController:
    def __init__(self, store, dispatcher, scan_use_case,
                 process_use_case, github_use_case, settings_use_case):
        self.store = store
        self.dispatcher = dispatcher
        self.scan_uc = scan_use_case
        self.process_uc = process_use_case
        self.github_uc = github_use_case
        self.settings_uc = settings_use_case

    def scan_only(self):
        state = self.store.state
        AsyncRuntime.run_coroutine(self.scan_uc.execute(state))

    def start_processing(self, target, save_path=None):
        state = self.store.state
        AsyncRuntime.run_coroutine(self.process_uc.execute(state, target, save_path))
```

### Нарушение 2: `FormattingService` — смешение ответственностей

```python
# ❌ ДО: один сервис делает 4 вещи
class FormattingService:
    def format_output(self): ...        # 1. координация форматов
    def _generate_tree(self): ...       # 2. генерация ASCII-дерева
    def _format_dependency_graph(self): # 3. форматирование графа
    def _render_custom_template(self):  # 4. рендер Jinja2
    def _to_xml(self): ...
    def _to_markdown(self): ...
    def _to_plain(self): ...
```

```python
# ✅ ПОСЛЕ: отдельный TreeBuilder, форматы — стратегии
class TreeBuilder:
    """Только генерация дерева"""
    def build(self, paths: List[str]) -> str: ...

class FormattingService:
    """Только координация форматирования"""
    def __init__(self, formatters: Dict[str, FormatterStrategy],
                 tree_builder: TreeBuilder): ...
```

### Нарушение 3: `Store._reducer` — монолитная функция

```python
# ❌ ДО: 60+ строк if-elif в одной функции
def _reducer(self, action_type, payload):
    if action_type == UI_SET_LOADING: ...
    elif action_type == UI_UPDATE_STATUS: ...
    elif action_type == SETTINGS_LOADED: ...
    # ... 20+ elif
```

```python
# ✅ ПОСЛЕ: реестр обработчиков
class Store:
    def __init__(self):
        self._handlers: Dict[str, Callable] = {}
        self._register_handlers()

    def _register_handlers(self):
        self._handlers[UI_SET_LOADING] = self._handle_set_loading
        self._handlers[SETTINGS_LOADED] = self._handle_settings_loaded
        # ...

    def _reducer(self, action_type, payload):
        handler = self._handlers.get(action_type)
        if handler:
            handler(payload)
```

---

## O — Open/Closed Principle

**Правило:** Классы открыты для расширения, закрыты для изменения.

### Нарушение 1: `IntegrationService` дублирует логику из `integration_strategies.py`

```python
# ❌ ДО: IntegrationService содержит всю Windows-логику инлайн.
# integration_strategies.py существует, но не используется.
# Чтобы добавить Linux — нужно менять IntegrationService.

class IntegrationService:
    def install_context_menu(self):
        # 50 строк Windows-кода прямо здесь
        key_dir = winreg.CreateKey(...)
        ...
```

```python
# ✅ ПОСЛЕ: IntegrationService использует стратегии
class IntegrationService:
    def __init__(self, strategy: ContextMenuStrategy):
        self._strategy = strategy  # подменяется через DI

    def install_context_menu(self, python_path=None):
        return self._strategy.install(python_path)

# DIContainer выбирает стратегию по платформе:
strategy = WindowsContextMenuStrategy() if platform.system() == "Windows" \
    else LinuxContextMenuStrategy()
integration_service = IntegrationService(strategy)
```

### Нарушение 2: `CleanerService._remove_comments` — жёсткий список расширений

```python
# ❌ ДО: добавить новый язык — изменить метод
@staticmethod
def _remove_comments(text, ext):
    if ext in ['.js', '.ts', ...]:
        text = re.sub(r'/\*.*?\*/', ...)
    elif ext in ['.py', '.sh', ...]:
        text = re.sub(r'#.*$', ...)
```

```python
# ✅ ПОСЛЕ: конфигурируемые правила
COMMENT_RULES: Dict[str, List[CommentRule]] = {
    'c_style': [CStyleBlockRule(), CStyleLineRule()],
    'hash_style': [HashLineRule()],
}
EXT_TO_RULE = {
    '.js': 'c_style', '.ts': 'c_style',
    '.py': 'hash_style', '.sh': 'hash_style',
}
```

---

## L — Liskov Substitution Principle

**Правило:** Подтипы должны быть полностью заменяемы своими базовыми типами.

### Нарушение: `DIContainer` создаёт `MainController` с неверной сигнатурой

```python
# ❌ ДО: в di_container.py
self.main_controller = MainController(
    self.store, self.dispatcher,
    self.scan_use_case, self.process_use_case, self.settings_repo
    # ← передаёт 5 аргументов
)

# Но в main_controller.py:
class MainController:
    def __init__(self, store: Store, dispatcher: Dispatcher):
        # ← принимает только 2 аргумента!
        # Код в DIContainer никогда не работал.
```

```python
# ✅ ПОСЛЕ: единственный и согласованный контракт
class MainController:
    def __init__(self, store: Store, dispatcher: Dispatcher,
                 scan_use_case: ScanWorkspaceUseCase,
                 process_use_case: ProcessWorkspaceUseCase,
                 github_use_case: GitHubUseCase,
                 settings_use_case: SettingsUseCase):
        ...
```

### Нарушение: `ProcessWorkspaceUseCase` ссылается на несуществующие action types

```python
# ❌ ДО: action types не определены в action_types.py
self.dispatcher.dispatch(WORKFLOW_STARTED, ...)   # NameError в runtime!
self.dispatcher.dispatch(LOG_MESSAGE_ADDED, ...)   # NameError в runtime!
self.dispatcher.dispatch(PROCESSING_COMPLETED, ...) # NameError в runtime!
```

```python
# ✅ ПОСЛЕ: все константы определены в action_types.py
# action_types.py:
WORKFLOW_STARTED = 'WORKFLOW_STARTED'
WORKFLOW_PROGRESS = 'WORKFLOW_PROGRESS'
WORKFLOW_FINISHED = 'WORKFLOW_FINISHED'
WORKFLOW_ERROR = 'WORKFLOW_ERROR'
```

---

## I — Interface Segregation Principle

**Правило:** Клиенты не должны зависеть от методов, которые они не используют.

### Нарушение: `CliController` создаёт все те же сервисы, что и `MainController`

```python
# ❌ ДО: оба контроллера независимо инстанцируют 10+ сервисов
class CliController:
    def __init__(self):
        self.settings_repo = SettingsRepository()
        self.fs_repo = FileSystemRepository()
        self.file_service = FileService(self.fs_repo)
        self.process_service = ProcessingService(self.fs_repo)
        self.cleaner_service = CleanerService()
        # ... ещё 7 сервисов
```

```python
# ✅ ПОСЛЕ: CliController получает только то, что ему нужно
class CliController:
    def __init__(self, settings_repo: SettingsRepository,
                 scan_use_case: ScanWorkspaceUseCase,
                 process_use_case: ProcessWorkspaceUseCase):
        self.settings_repo = settings_repo
        self.scan_uc = scan_use_case
        self.process_uc = process_use_case
```

---

## D — Dependency Inversion Principle

**Правило:** Высокоуровневые модули не зависят от низкоуровневых. Оба зависят от абстракций.

### Нарушение: контроллеры сами создают зависимости (`new` в середине кода)

```python
# ❌ ДО: прямое инстанцирование — жёсткая связь, невозможно тестировать
class MainController:
    def __init__(self, store, dispatcher):
        self.fs_repo = FileSystemRepository()   # ← нарушение DIP
        self.file_service = FileService(self.fs_repo)
        self.process_service = ProcessingService(self.fs_repo)
```

```python
# ✅ ПОСЛЕ: зависимости инжектируются снаружи
class MainController:
    def __init__(self, store: Store, dispatcher: Dispatcher,
                 scan_uc: ScanWorkspaceUseCase,
                 process_uc: ProcessWorkspaceUseCase,
                 ...):
        # Все зависимости получены через конструктор
        # Легко подменить на моки в тестах
```

### Нарушение: `main.py` не использует `DIContainer`

```python
# ❌ ДО: main.py вручную создаёт часть зависимостей
store = Store()
dispatcher = Dispatcher(store)
controller = MainController(store, dispatcher)  # DIContainer игнорируется
```

```python
# ✅ ПОСЛЕ: единая точка сборки
container = DIContainer()
app = MainWindow(container.store, container.main_controller)
```
