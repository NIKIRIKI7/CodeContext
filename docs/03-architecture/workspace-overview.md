# Обзор воркспейса и Архитектурные границы

Архитектура проекта объединяет **Ports & Adapters (Hexagonal / Onion Architecture)** из Главы 8 книги Влада Хононова и принцип **Deep Modules (Глубоких модулей Джона Остерхаута)** из Главы 14.

---

## 1. Роли 8 крейтов воркспейса

| Крейт | Слой архитектуры | Разрешённые локальные зависимости | Назначение |
| :--- | :--- | :--- | :--- |
| **`core`** | Domain + Application Core | *Нет (`[]`)* | Чистая доменная логика, разделённая на поддомены (`shared_kernel`, `packing`, `applying`, `preset`, `system`). |
| **`scanner`** | Infrastructure Adapter (ACL) | `core` | Параллельный обход ФС (`ignore`, `rayon`), детекция бинарников (`memchr`), потоковое чтение и минификация. |
| **`apply`** | Infrastructure Adapter (ACL) | `core` | Парсинг ответов ИИ, автофикс, генерация diff (`similar`), песочница, атомарная запись и бэкапы. |
| **`config`** | Infrastructure Adapter (ACL) | `core` | Сериализация JSON (`serde`), каскадное чтение настроек и миграции версий схемы. |
| **`platform`** | Infrastructure Adapter (ACL) | `core` | Адаптеры ОС: буфер обмена (`arboard`), глобальные хоткеи, Single-Instance IPC и меню Проводника/Finder. |
| **`di`** | Composition Root | `core`, `config`, `scanner`, `apply`, `platform` | Контейнер внедрения зависимостей, связывающий абстрактные порты `core` с конкретными адаптерами. |
| **`ui`** | Presentation Layer | `core`, `di` | Графический интерфейс на `gpui` + `gpui-component` по паттерну `State-Entity-View`. |
| **`app`** | Bootstrap / Entrypoint | `core`, `di`, `ui`, `platform`, `config` | Точка входа бинарника: роутинг CLI-аргументов (`cli.rs`), быстрый `headless`-режим и запуск окна GPUI. |

---

## 2. Автоматический контроль архитектуры (`architecture.rs`)
Все архитектурные правила проверяются автоматически при каждом запуске `cargo test --package app --test architecture`:
1. **`test_1_ports_and_adapters_workspace_boundaries`** — проверяет белый список прямых и транзитивных зависимостей между крейтами.
2. **`test_2_generic_subdomain_acl_isolation`** — гарантирует, что `gpui`, `ignore`, `similar`, `arboard` не протекают в `core` или соседние адаптеры.
3. **`test_3_workspace_lints_enforced_everywhere`** — проверяет наличие `[lints] workspace = true` во всех `Cargo.toml`.
4. **`test_4_ddd_core_subdomains_and_context_map`** — запрещает технические папки в корне `core/src` и проверяет изоляцию поддоменов по Карте Контекстов.
5. **`test_5_ddd_tactical_building_blocks_invariants`** — проверяет неизменяемость Value Objects (запрет `&mut self`) и направление зависимостей `values` <- `aggregates` <- `ports` <- `services`.
6. **`test_6_gpui_state_entity_view_conventions`** — защищает паттерн `State-Entity-View` в `crates/ui`.
7. **`test_7_deep_modules_encapsulation_and_acyclic_graph`** — запускает `cargo-machete`, `cargo-modules` и `cargo-public-api`.
8. **`test_8_max_files_per_directory_cohesion`** — запрещает накапливать более **4 файлов реализации** в любой папке `src/`.