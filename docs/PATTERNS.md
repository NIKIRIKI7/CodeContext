# Паттерны проектирования в CodeContext

## Используемые паттерны

### 1. Redux (State Management)
**Где:** `src/store/`

Однонаправленный поток данных. `Store` хранит единственный источник истины (`AppState`).
Изменения только через `dispatch(action_type, payload)`. UI подписывается на изменения.

```
Action → Dispatcher → Store.reducer → State → notify → UI
```

**Зачем:** Предсказуемость состояния, лёгкий дебаггинг (логи actions), отсутствие «скрытых» мутаций.

---

### 2. Strategy (Паттерн Стратегия)
**Где:** `src/services/strategies/`

Три места применения:

| Контекст | Интерфейс | Реализации |
|---|---|---|
| `DependencyService` | `DependencyParserStrategy` | `PythonDependencyParser`, `WebDependencyParser` |
| `ImportResolutionService` | `ImportResolutionStrategy` | `StandardImportStrategy`, `FSDImportStrategy`, `DDDImportStrategy`, `AtomicDesignImportStrategy` |
| `IntegrationService` | `ContextMenuStrategy` | `WindowsContextMenuStrategy`, `LinuxContextMenuStrategy`, `MacOSContextMenuStrategy` |

**Зачем:** Добавление нового языка/ОС/архитектуры — только новый класс, без изменения существующего кода (OCP).

---

### 3. Repository (Репозиторий)
**Где:** `src/data/`

Абстрагирует хранилище данных от бизнес-логики. Сервисы работают с репозиторием, не зная — файловая система это, база данных или in-memory.

```python
class FileSystemRepository:
    async def read_file_async(self, path: str) -> Optional[str]: ...
    async def walk_directory_async(self, path, ignored_dirs, extensions): ...
```

**Зачем:** Возможность заменить хранилище (например, для тестов — in-memory репозиторий).

---

### 4. Observer (Наблюдатель)
**Где:** `Store.subscribe()` → `MainWindow._on_store_changed`

UI подписывается на Store и реагирует на каждое изменение состояния. Отписка возвращается как `unsubscribe` функция (функциональный Observer).

```python
self.unsubscribe = self.store.subscribe(self._on_store_changed_threadsafe)
# При закрытии:
self.unsubscribe()
```

**Зачем:** Полная развязка UI и бизнес-логики. UI не знает, кто изменил состояние.

---

### 5. Command (Команда) — через Action Types
**Где:** `src/actions/action_types.py`

Каждое действие (`SCAN_START`, `PROCESSING_SUCCESS`, etc.) — это «команда» для изменения состояния. Команды сериализуемы, логгируемы, воспроизводимы.

---

### 6. Facade (Фасад)
**Где:** `DIContainer`, `MainController`

`MainController` — фасад над системой Use Cases. UI взаимодействует только с контроллером, не зная о внутренней сложности (Use Cases → Services → Repositories).

---

### 7. Template Method (Шаблонный метод)
**Где:** `PipelineUtils.process_files_batch`

Фиксированный алгоритм: `clean → skeleton (optional) → count_tokens`. Конкретные шаги делегируются сервисам.

---

### 8. Dependency Injection Container
**Где:** `src/di_container.py`

Централизованная сборка графа объектов. Решает проблему «кто создаёт кого» без жёсткой связи между классами.

```python
# Единственное место с `new`:
class DIContainer:
    def __init__(self):
        self.fs_repo = FileSystemRepository()
        self.file_service = FileService(self.fs_repo)  # инжектируем fs_repo
        ...
```

---

## Антипаттерны, которые были устранены

### ❌ God Object (Бог-объект)
`MainController` в исходнике знал всё и делал всё. **Устранён** разбиением на Use Cases.

### ❌ Service Locator
Контроллеры сами создавали сервисы внутри `__init__`. **Устранён** инжекцией зависимостей через конструктор.

### ❌ Feature Envy (Зависть к возможностям)
`IntegrationService` содержал код, который уже был реализован в `integration_strategies.py`. **Устранён** делегированием стратегии.

### ❌ Dead Code (Мёртвый код)
`scan_use_case.py` и `github_use_case.py` были пустыми файлами, `DIContainer` создавался, но не использовался в `main.py`. **Устранён** реализацией и подключением.

### ❌ Primitive Obsession
`action_types` — строки-константы разбросаны по коду без единого источника. **Устранён** обязательным импортом из `action_types.py`.
