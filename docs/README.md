# Документация проекта CodeContext

**CodeContext** — кроссплатформенное десктопное приложение на **Rust + GPUI** для сверхбыстрой упаковки исходного кода в буфер обмена и безопасного детерминированного применения ответов ИИ-чатов к файлам проекта (без использования платных API и встроенных LLM).

---

## Навигация по разделам

### 1. Продукт и требования (`01-product/`)
Отвечает на стратегические вопросы *«Что мы строим?»* и *«Зачем?»*:
* [spec.md](./01-product/spec.md) — Цели продукта, критерии успеха, функциональные и нефункциональные требования.
* [scenarios.md](./01-product/scenarios.md) — Пользовательские сценарии (A1–A3, B1–B2) и приёмочные критерии (Gherkin).
* [roadmap.md](./01-product/roadmap.md) — Дорожная карта (MVP, Этапы 2–3) и матрица технических рисков.

### 2. Доменная модель DDD (`02-domain/`)
Фиксирует знания предметной области по методологии *Domain-Driven Design*:
* [ubiquitous-language.md](./02-domain/ubiquitous-language.md) — Глоссарий Единого Языка (Ubiquitous Language) без технического жаргона.
* [subdomains.md](./02-domain/subdomains.md) — Декомпозиция на Core, Supporting и Generic поддомены и выбор тактических паттернов.
* [context-map.md](./02-domain/context-map.md) — Карта ограниченных контекстов (Context Map), Shared Kernel и Anticorruption Layers.

### 3. Архитектура и конвенции (`03-architecture/`)
Отвечает на тактический вопрос *«Как устроена система в коде?»*:
* [workspace-overview.md](./03-architecture/workspace-overview.md) — Структура 8 крейтов воркспейса, Ports & Adapters и фитнес-функции `architecture.rs`.
* [gpui-state-entity-view.md](./03-architecture/gpui-state-entity-view.md) — Паттерн `State-Entity-View`, проекции чтения (CQRS) и виртуализация 4 МБ текста в GPUI.
* [security-and-performance.md](./03-architecture/security-and-performance.md) — Песочница путей, защита секретов и доказательство 1 аллокации кучи (`dhat`).
* [build-and-test.md](./03-architecture/build-and-test.md) — Сборка максимальной производительности, тестирование и PGO.

#### Журнал архитектурных решений (`03-architecture/adr/`)
* [ADR-001: Вертикальные поддомены DDD и Ports & Adapters](./03-architecture/adr/001-onion-and-ddd-slices.md)
* [ADR-002: Двухфазное сканирование и сборка буфера с нулём реаллокаций](./03-architecture/adr/002-zero-realloc-packing.md)
* [ADR-003: Транзакционное применение патчей (Staging + Backup Manifest)](./03-architecture/adr/003-transactional-apply.md)

### 4. Справочные материалы (`04-references/`)
* `drafts/` — Исходные черновики требований (`Описание_проекта.docx`).
* `articles/` — Учебные материалы и руководства по GPUI и DDD.