# Ponytail Debt Ledger

This file tracks all deliberate `ponytail:` shortcuts, their ceilings, and upgrade paths across the codebase.

## main.py
- `main.py:145 — Встроенный луп событий PySide6 вместо самописного фонового потока. ceiling: none. upgrade: none. [no-trigger]`

## src/services/integration_service.py
- `src/services/integration_service.py:227 — Mac Automator actions integration (simplified from original). ceiling: none. upgrade: none. [no-trigger]`

## src/services/output_service.py
- `src/services/output_service.py:23 — stdlib webbrowser delegates to the native OS file/folder openers. ceiling: none. upgrade: none. [no-trigger]`

## src/services/patch_service.py
- `src/services/patch_service.py:39 — Обычный str.find() в 1000 раз быстрее медленного difflib.SequenceMatcher. ceiling: none. upgrade: none. [no-trigger]`

## src/services/plugin_manager.py
- `src/services/plugin_manager.py:80 — Песочницы не работают в Python. Отдаем контроллер плагинам напрямую. ceiling: none. upgrade: none. [no-trigger]`

## src/services/skeleton_service.py
- `src/services/skeleton_service.py:55 — merged identical visit_FunctionDef and visit_AsyncFunctionDef logic, deleted redundant visit_ClassDef. ceiling: none. upgrade: none. [no-trigger]`

## src/ui/main_window.py
- `src/ui/main_window.py:264 — fpdf was dropped. explicitly filtering md/txt so users don't save unopenable .pdf files. ceiling: none. upgrade: none. [no-trigger]`

## src/use_cases/patch_use_case.py
- `src/use_cases/patch_use_case.py:15 — llms always wrap json in code blocks. if no blocks, try the whole string as fallback. ceiling: none. upgrade: none. [no-trigger]`

## src/use_cases/process_use_case.py
- `src/use_cases/process_use_case.py:19 — Отказ от mmap(). Дисковый кэш ОС делает read_text() достаточно быстрым. ceiling: none. upgrade: none. [no-trigger]`

## src/utils/pipeline_utils.py
- `src/utils/pipeline_utils.py:57 — queue.pop(0) was O(n). Optimized with collections.deque.popleft for O(1) performance. ceiling: queue.pop(0) was O(n). upgrade: collections.deque.popleft for O(1) performance.`

---

10 markers, 9 with no trigger.
