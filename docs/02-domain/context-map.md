# Карта контекстов (Context Map)

В соответствии с **Главой 4 книги «Learning Domain-Driven Design»**, данный документ определяет границы моделей и паттерны интеграции между поддоменами внутри ядра и внешними инфраструктурными адаптерами.

---

## 1. Схема интеграции (Context Map)

```text
+-----------------------------------------------------------------------------------+
|                                 crates/core                                       |
|                                                                                   |
|   +---------------------------------------------------------------------------+   |
|   |                       shared_kernel (Shared Kernel)                       |   |
|   |            [FileKind, LineEnding, Anomaly, CancelToken]                   |   |
|   +---------------------------------------------------------------------------+   |
|             ^                           ^                           ^             |
|             |                           |                           |             |
|   +-------------------+       +-------------------+       +-------------------+   |
|   |   preset (U)      |       |   packing (D)     |       |   applying (D)    |   |
|   |  (Supporting)     |<------|      (Core)       |       |      (Core)       |   |
|   | [Preset, Template]|  C-S  | [Project, Chunker]|       | [ApplyPlan, Undo] |   |
|   +-------------------+       +-------------------+       +-------------------+   |
|             ^                           ^                           ^             |
+-------------|---------------------------|---------------------------|-------------+
              | Ports                     | Ports                     | Ports
        +-----+-----+               +-----+-----+               +-----+-----+
        |    ACL    |               |    ACL    |               |    ACL    |
        +-----+-----+               +-----+-----+               +-----+-----+
              |                           |                           |
      +---------------+           +---------------+           +---------------+
      | crates/config |           |crates/scanner |           | crates/apply  |
      | (serde_json)  |           | (ignore/rayon)|           |  (similar)    |
      +---------------+           +---------------+           +---------------+
```

---

## 2. Паттерны интеграции между контекстами

### 2.1. `shared_kernel` (Общее ядро — Глава 4, стр. 50)
* **Участники:** Все поддомены внутри `crates/core` (`packing`, `applying`, `preset`, `system`).
* **Что входит:** Только базовые неизменяемые примитивы (`FileKind`, `LineEnding`, `Anomaly`, `CancelToken`).
* **Правило защиты (проверяется `test_4` в `architecture.rs`):** `shared_kernel` **не имеет права** импортировать ничего из `packing`, `applying`, `preset` или `system`.

### 2.2. `Customer–Supplier` (Поставщик — Потребитель, Глава 4, стр. 53)
* **Upstream (Поставщик):** поддомен `core::preset` (предоставляет модели `Preset` и `OutputTemplate`).
* **Downstream (Потребители):** Core-поддомены `core::packing` и `core::applying`.
* **Правило изоляции (`Separate Ways` между Core-поддоменами):** `core::packing` и `core::applying` **полностью изолированы друг от друга**.

### 2.3. `Anticorruption Layer (ACL)` (Антикоррупционный слой — Глава 4, стр. 54)
Каждый инфраструктурный крейт выступает в роли **Stateless Model Translation Proxy** (Глава 9, стр. 138):
* `crates/config` переводит `Dto`-структуры JSON (`schema/dtos.rs`) в чистые доменные модели `core::preset` через `schema/mappers.rs`.
* `crates/scanner` переводит `ignore::DirEntry` в доменный `FileEntry` / `ScannedFile`.
* `crates/apply` переводит сырой текст ответа ИИ и ханки `similar::TextDiff` в доменный `ApplyPlan`.