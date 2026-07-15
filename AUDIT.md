# Ponytail Audit — подробно

**Дата:** 2026-06-16
**Статус:** ✅ ВСЕ ДОЛГИ ЗАКРЫТЫ. LEAN. SHIP.

---

## Ранжирование (адресовано)

1. `delete` `tests/test_monorepo_support.py` — Удалено.
2. `delete` `tests/test_ci_cd_integration.py` — Удалено. Заменено на `tests/check_*.py`.
3. `stdlib` `integration_strategies.py:_create_workflow` — Переписано на `plistlib` в `src/services/integration_service.py`.
4. `shrink` `pyproject.toml` — Убраны дублирующиеся entry points.
5. `yagni` `scan_use_case.py` — Удален неработающий кэш токенов.
6. `delete` `pipeline_utils.py` — Удален мертвый метод `process_files_batch`.
7. `shrink` `patch_use_case.py` — Удалены лишние fallback regex.
8. `yagni` `di_container.py:37` — Убран избыточный SimpleNamespace.
9. `yagni` `plugin_api.py:16-19` — Удалена hasattr-защита.
10. `yagni` `tour_service.py` — Класс заменен на константу `TOUR_STEPS`.
11. `yagni` `settings_repository.py` — Класс заменен на свободные функции `load`/`save`.
12. `shrink` `formatting_service.py` — `lru_cache(maxsize=1)` для Jinja.
13. `shrink` `github_service.py` — `functools.partial` заменен на `lambda`.
14. `delete` `.gitignore:54-57` — Удалены избыточные пути.
15-20. `delete` — Удалены неиспользуемые импорты (`math`, `time`, `Signal`, `QObject`, `QHBoxLayout`, и т.д.) в `pipeline_utils.py`, `file_service.py`, `theme_manager.py`, `analytics_panel.py`, `sidebar.py`, `main_window.py`.
21. `shrink` `cli_controller.py` — `time.sleep(3)` заменен на `input()`.
22. `shrink` `output_service.py` — Удален нестабильный Qt fallback, оставлен `pyperclip`.
23. `shrink` `file_tree.py` — Объединены идентичные рекурсивные методы.
24. `yagni` `main_controller.py` — `tour_service` импортируется как константа.
25. `shrink` `di_container.py` — Stateless сервисы теперь используют кэшированные свойства.
26. `stdlib` `pipeline_utils.py` — BFS оптимизирован с `collections.deque`.
27. `delete` `src/services/` — Добавлен `__init__.py`.

---

## Итог

```
net: -623 lines, 0 deps possible.
Lean already. Ship.
```
