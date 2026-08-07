# Ponytail Debt Ledger

This ledger tracks all deliberate `ponytail:` shortcuts, their ceilings, and upgrade paths across the codebase.

## main.py
- main.py:145 — Встроенный луп событий PySide6 вместо самописного фонового потока. ceiling: Встроенный луп событий PySide6 вместо самописного фонового потока. upgrade: None. [no-trigger]

## src/services/integration_service.py
- src/services/integration_service.py:227 — Mac Automator actions integration (simplified from original). ceiling: Mac Automator actions integration (simplified from original). upgrade: None. [no-trigger]

## src/services/output_service.py
- src/services/output_service.py:23 — stdlib webbrowser delegates to the native OS file/folder openers. ceiling: stdlib webbrowser delegates to the native OS file/folder openers. upgrade: None. [no-trigger]

## src/services/patch_service.py
- src/services/patch_service.py:39 — Обычный str.find() в 1000 раз быстрее медленного difflib.SequenceMatcher. ceiling: Обычный str.find() в 1000 раз быстрее медленного difflib.SequenceMatcher. upgrade: None. [no-trigger]

## src/services/plugin_manager.py
- src/services/plugin_manager.py:80 — Песочницы не работают в Python. Отдаем контроллер плагинам напрямую. ceiling: Песочницы не работают в Python. Отдаем контроллер плагинам напрямую. upgrade: None. [no-trigger]

## src/services/skeleton_service.py
- src/services/skeleton_service.py:55 — merged identical visit_FunctionDef and visit_AsyncFunctionDef logic. ceiling: merged identical visit_FunctionDef and visit_AsyncFunctionDef logic. upgrade: deleted redundant visit_ClassDef.

## src/ui/main_window.py
- src/ui/main_window.py:264 — fpdf was dropped. explicitly filtering md/txt so users don't save unopenable .pdf files. ceiling: fpdf was dropped. explicitly filtering md/txt so users don't save unopenable .pdf files. upgrade: None. [no-trigger]

## src/use_cases/patch_use_case.py
- src/use_cases/patch_use_case.py:15 — llms always wrap json in code blocks. if no blocks. ceiling: llms always wrap json in code blocks. if no blocks. upgrade: try the whole string as fallback.

## src/use_cases/process_use_case.py
- src/use_cases/process_use_case.py:19 — Отказ от mmap(). Дисковый кэш ОС делает read_text() достаточно быстрым. ceiling: Отказ от mmap(). Дисковый кэш ОС делает read_text() достаточно быстрым. upgrade: None. [no-trigger]

## src/utils/pipeline_utils.py
- src/utils/pipeline_utils.py:57 — queue.pop(0) was O(n). Optimized with collections.deque.popleft for O(1) performance. ceiling: queue.pop(0) was O(n). Optimized with collections.deque.popleft for O(1) performance. upgrade: None. [no-trigger]

10 markers, 8 with no trigger.
