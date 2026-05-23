# CodeContext — Архитектурное руководство

## Обзор

CodeContext построен по принципу **Clean Architecture** с элементами **Redux-паттерна** для управления состоянием. Слои разделены по зоне ответственности и направлению зависимостей: зависимости всегда смотрят **внутрь**, к доменному ядру.

```
┌──────────────────────────────────────────────────┐
│                   UI / CLI                       │  Presentation Layer
│         (main_window, cli_controller)            │
├──────────────────────────────────────────────────┤
│              Controllers / Use Cases             │  Application Layer
│    (main_controller, scan_use_case, etc.)        │
├──────────────────────────────────────────────────┤
│                  Services                        │  Domain Layer
│  (file_service, processing_service, etc.)        │
├──────────────────────────────────────────────────┤
│              Data / Repositories                 │  Infrastructure Layer
│   (file_system_repository, settings_repo)        │
└──────────────────────────────────────────────────┘
```

---

## Слои и их правила

### 1. Infrastructure (Data)
**Папка:** `src/data/`

- Работает с файловой системой, реестром ОС, JSON-файлами.
- **Не знает** ни о каких сервисах или use cases.
- Все блокирующие I/O операции обёрнуты в `asyncio.to_thread`.
- Репозитории возвращают примитивы (`str`, `List[str]`, `Dict`) или `None`.

**Правило:** Репозиторий — это адаптер к внешнему миру. Он не содержит бизнес-логики.

### 2. Domain (Services)
**Папка:** `src/services/`

- Содержит бизнес-логику, изолированную от конкретной транспортной инфраструктуры.
- Каждый сервис отвечает ровно за **одну** задачу.
- Принимает зависимости через конструктор (DIP).
- Для вариативного поведения использует **паттерн Стратегия** (`src/services/strategies/`).

**Правило:** Сервис не знает о Store, Dispatcher, UI и контроллерах.

### 3. Application (Use Cases)
**Папка:** `src/use_cases/`

- Оркестрирует сервисы для выполнения одного бизнес-сценария.
- Принимает `Dispatcher` для отправки событий в Store.
- **Не содержит** бизнес-логики — только координацию.
- Возвращает `None`; результат передаётся через `Dispatcher`.

**Правило:** Один Use Case — один бизнес-сценарий. Если сценарий разрастается, разбить на два.

### 4. Application (Controllers)
**Папка:** `src/controllers/`

- Принимают события от UI/CLI и вызывают соответствующие Use Cases.
- **Не содержат** бизнес-логики и не вызывают сервисы напрямую.
- Владеют только `Store`, `Dispatcher` и Use Cases.

**Правило:** Контроллер — тонкий. Если метод длиннее 15 строк, логика должна переехать в Use Case.

### 5. Presentation (UI)
**Папка:** `src/ui/`

- Компоненты реагируют на изменения `AppState` через подписку на `Store`.
- Компоненты **не вызывают** сервисы и репозитории напрямую.
- Все пользовательские действия делегируются контроллеру.

**Правило:** UI — это «вид на State». Никакой логики, только отображение и делегирование.

---

## Поток данных (Unidirectional Data Flow)

```
User Action
    │
    ▼
Controller.method()
    │
    ▼
UseCase.execute(state)
    │  (читает текущий state из Store)
    ▼
Services (бизнес-логика)
    │
    ▼
Dispatcher.dispatch(ACTION, payload)
    │
    ▼
Store._reducer(action, payload)
    │  (обновляет AppState)
    ▼
Store._notify() → подписчики
    │
    ▼
UI.update(new_state)
```

Данные текут в **одном направлении**. UI никогда не мутирует State напрямую.

---

## Управление состоянием (Redux-like Store)

### AppState
Неизменяемый снимок состояния. `Store.state` всегда возвращает **глубокую копию**.

### Action Types
Все строки-константы для действий определены **только** в `src/actions/action_types.py`. Запрещено использовать строки-литералы напрямую в коде.

### Reducer
`Store` использует **реестр редьюсеров** (словарь `{action_type: handler_func}`), а не цепочку `if-elif`. Добавление нового действия — регистрация новой функции без изменения существующих.

### Dispatcher
Тонкая обёртка над `Store.dispatch`. Передаётся в Use Cases и Controllers через DI.

---

## Внедрение зависимостей (DIContainer)

**Файл:** `src/di_container.py`

`DIContainer` — единственное место, где создаются конкретные классы. Он собирает граф зависимостей и передаёт объекты через конструкторы.

**Правила DI:**
1. Никаких `SomeService()` внутри других сервисов или контроллеров.
2. Зависимости объявляются в `__init__` через типы (желательно абстрактные).
3. `DIContainer` создаётся **один раз** в `main.py`.

---

## Паттерн Стратегия в сервисах

Используется в трёх местах:
- `DependencyService` → `DependencyParserStrategy` (Python/Web)
- `ImportResolutionService` → `ImportResolutionStrategy` (Standard/FSD/DDD/Atomic)
- `IntegrationService` → `ContextMenuStrategy` (Windows/Linux/macOS)

**Правило:** Если поведение зависит от типа файла, ОС или конфигурации — использовать Стратегию, а не `if/elif`.

---

## Асинхронность

- `AsyncRuntime` запускает `asyncio` event loop в отдельном потоке (совместимость с Tkinter).
- Блокирующие CPU-операции запускаются через `asyncio.to_thread`.
- I/O-операции — нативные `async/await`.
- **Запрещено** вызывать `asyncio.run()` в методах, которые вызываются из контекста уже запущенного loop.

---

## Структура файлов

```
CodeContext/
├── main.py                    # Точка входа. Создаёт DIContainer.
├── docs/                      # Архитектурная документация
│   ├── ARCHITECTURE.md
│   ├── SOLID.md
│   ├── PATTERNS.md
│   └── DECISIONS.md
└── src/
    ├── actions/
    │   ├── action_types.py    # Все константы действий
    │   └── dispatcher.py      # Тонкая обёртка над Store.dispatch
    ├── controllers/
    │   ├── main_controller.py # GUI-контроллер (тонкий)
    │   └── cli_controller.py  # CLI-контроллер (тонкий)
    ├── data/
    │   ├── file_system_repository.py
    │   └── settings_repository.py
    ├── di_container.py        # Граф зависимостей
    ├── services/
    │   ├── strategies/        # Паттерн Стратегия
    │   ├── cleaner_service.py
    │   ├── dependency_service.py
    │   ├── file_service.py
    │   ├── formatting_service.py
    │   ├── github_service.py
    │   ├── import_resolution_service.py
    │   ├── integration_service.py
    │   ├── output_service.py
    │   ├── processing_service.py
    │   ├── skeleton_service.py
    │   └── token_service.py
    ├── store/
    │   ├── state.py           # AppState, AppSettings, ProcessedFile
    │   └── store.py           # Store с реестром редьюсеров
    ├── ui/
    │   ├── components/        # Атомарные UI-компоненты
    │   ├── dialogs.py
    │   └── main_window.py
    ├── use_cases/
    │   ├── scan_use_case.py   # Сценарий: сканирование файлов
    │   ├── process_use_case.py # Сценарий: обработка и экспорт
    │   └── github_use_case.py  # Сценарий: клонирование репо
    └── utils/
        ├── async_runtime.py
        ├── config.py
        ├── logger.py
        └── pipeline_utils.py
```
