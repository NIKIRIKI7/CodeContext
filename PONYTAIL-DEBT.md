main.py:145 — Встроенный луп событий PySide6 вместо самописного фонового потока. ceiling: N/A. upgrade: N/A. no-trigger
src/services/integration_service.py:227 — Mac Automator actions integration (simplified from original). ceiling: N/A. upgrade: N/A. no-trigger
src/services/output_service.py:23 — stdlib webbrowser delegates to the native OS file/folder openers. ceiling: N/A. upgrade: N/A. no-trigger
src/services/patch_service.py:39 — Обычный str.find() в 1000 раз быстрее медленного difflib.SequenceMatcher.. ceiling: N/A. upgrade: N/A. no-trigger
src/services/plugin_manager.py:80 — Песочницы не работают в Python. Отдаем контроллер плагинам напрямую.. ceiling: N/A. upgrade: N/A. no-trigger
src/services/skeleton_service.py:55 — merged identical visit_FunctionDef and visit_AsyncFunctionDef logic. ceiling: merged identical visit_FunctionDef and visit_AsyncFunctionDef logic. upgrade: deleted redundant visit_ClassDef.
src/ui/main_window.py:264 — fpdf was dropped. explicitly filtering md/txt so users don't save unopenable .pdf files.. ceiling: N/A. upgrade: N/A. no-trigger
src/use_cases/patch_use_case.py:15 — llms always wrap json in code blocks. if no blocks. ceiling: llms always wrap json in code blocks. if no blocks. upgrade: try the whole string as fallback..
src/use_cases/process_use_case.py:19 — Отказ от mmap(). Дисковый кэш ОС делает read_text() достаточно быстрым.. ceiling: N/A. upgrade: N/A. no-trigger
src/utils/pipeline_utils.py:57 — queue.pop(0) was O(n). Optimized with collections.deque.popleft for O(1) performance.. ceiling: N/A. upgrade: N/A. no-trigger

10 markers, 8 with no trigger.
