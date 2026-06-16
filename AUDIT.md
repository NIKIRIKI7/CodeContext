# Ponytail Audit

> Repo-wide scan for over-engineering. Ranked biggest savings first.
> Generated: 2026-06-16
> Metric: `net: -1450 lines, -2 deps possible.`

---

## 1. Redux store/dispatcher/actions/state — ~500 lines of event plumbing

**Files:** `src/store/store.py` (263), `src/store/state.py` (104), `src/actions/action_types.py` (58), `src/actions/dispatcher.py` (41)

**Problem:** The app implements a hand-rolled Redux pattern in Python: action type constants, a `Store` with registered handlers, a `Dispatcher` wrapper with throttling, a `State` dataclass, and a deep-copy snapshot mechanism. The single consumer is the PySide6 UI.

**Replacement:** Qt signals/slots or direct method calls on the controller. PySide6 already provides `QObject` signals for decoupled communication — no store, no dispatcher, no action constants, no snapshot copies needed.

```
# Current: 500+ lines, 4 files
dispatcher.dispatch(FOLDER_ADD, path)
# → store._handlers['FOLDER_ADD'](path)
# → store._state.selected_folders.append(path)
# → deepcopy → notify listeners

# Replacement: direct signal or call
controller.add_folder(path)
# → selected_folders.append(path)
# → emit signal if needed
```

**Savings:** ~420 lines net (some state models remain for persistence).

---

## 2. PipelineUtils ProcessPoolExecutor + temp file IPC — 156 lines

**File:** `src/utils/pipeline_utils.py`

**Problem:** `process_files_batch_parallel` spawns a `ProcessPoolExecutor`, chunks files, writes each processed file to a temp file on disk (`tempfile.mkstemp`), then the parent reads them all back and deletes them. The worker re-imports `CleanerService`, `SkeletonService`, `TokenService` from scratch per-chunk.

```
# Temp file dance:
fd, temp_path = tempfile.mkstemp(prefix="cc_ipc_", suffix=".txt")
with os.fdopen(fd, 'w') as f:
    f.write(cleaned)
# ... later:
with open(d["temp_file"], 'r') as f:
    content = f.read()
os.remove(d["temp_file"])
```

**Replacement:** `ThreadPoolExecutor` with shared memory — the services are CPU-light (regex, string ops), GIL-safe, and there is zero reason to serialize through disk.

**Savings:** ~60 lines, no temp file I/O, no worker re-imports.

---

## 3. IntegrationService — 52-line pure delegate facade

**File:** `src/services/integration_service.py`

**Problem:** Every method is a one-line delegation:

```python
def install_context_menu(self, ...): return self._strategy.install(...)
def remove_context_menu(self, ...): return self._strategy.remove(...)
def install_cli(self, ...): return self._strategy.install_cli(...)
def remove_cli(self, ...): return self._strategy.remove_cli(...)
```

**Replacement:** Call `_make_integration_strategy()` at the call site and use the result directly. The facade adds nothing.

**Savings:** ~50 lines.

---

## 4. DIContainer eagerly instantiates all 30+ objects — 138 lines

**File:** `src/di_container.py`

**Problem:** `DIContainer.__init__` creates every service, use case, controller, repository, and plugin API upfront — regardless of CLI vs GUI mode. Many objects take 5-12 constructor args, all wired by hand. Changes require editing the container.

**Replacement:** `@property`-based lazy init or module-level factory functions. Most services are stateless and can be created on demand.

**Savings:** ~80 lines.

---

## 5. FileSystemRepository dual sync/async methods — 141 lines

**File:** `src/data/file_system_repository.py`

**Problem:** Every I/O method has both a sync and async variant:

- `_read_file_sync` + `read_file_async`
- `_delete_directory_sync` + `delete_directory_async`
- `_walk_directory_sync` + `walk_directory_async`
- `get_git_status_async` (async-only, sync never needed)

The async variants all do `await asyncio.to_thread(self._sync_method, ...)`.

**Replacement:** Keep only sync methods, call `asyncio.to_thread` at the single call site in the use case. Reduces method count by half.

**Savings:** ~50 lines.

---

## 6. IPlugin ABC with one implementation — 12 lines

**File:** `src/api/plugin_api.py:55-65`

**Problem:** Abstract base class with one abstract method (`on_init`) and one default-noop (`on_shutdown`). One implementation exists (`hello_world` plugin) — it's the example.

**Replacement:** Document the protocol. No ABC needed until a second plugin exists. `on_shutdown` with empty default is already not abstract.

**Savings:** ~12 lines.

---

## 7. UIRegistry class — 24-line dict wrapper

**File:** `src/api/plugin_api.py:5-23`

**Problem:** `UIRegistry` holds two dicts and has two two-line methods (`register_sidebar_tab`, `register_action_button`). No behavior beyond `dict[key] = value`.

**Replacement:** `SimpleNamespace(sidebar_tabs={}, action_buttons={})`.

**Savings:** ~22 lines.

---

## 8. FormattingService — all static methods, no state

**File:** `src/services/formatting_service.py` (363 lines)

**Problem:** Every method is `@staticmethod` — `_to_markdown`, `_to_plain`, `_to_xml`, `_to_jsonl_mini`, `_to_jsonl_chunks`, `_generate_tree`, `generate_html_diff`, etc. The class is a namespace, not a class. `_env_cache` is the only class-level state, and it's a cache for `_render_custom_template`.

**Replacement:** Module-level functions. If `_env_cache` matters, keep it as `functools.lru_cache` on the template function.

**Savings:** ~5 lines (class keyword + docstring).

---

## 9. CleanerService, SkeletonService, TokenService, OutputService — stateless one-use classes

**Files:** `src/services/cleaner_service.py` (79), `src/services/skeleton_service.py` (105), `src/services/token_service.py` (30), `src/services/output_service.py` (70)

**Problem:** Each is a class with one public method (or a few), no instance state, called from a single use case. `CleanerService.clean()`, `SkeletonService.make_skeleton()`, `TokenService.count_tokens()`, `OutputService.copy_to_clipboard()` etc.

**Replacement:** Module-level functions. `TokenService.__init__` tries 3 fallbacks and caches an encoding — extract to `@functools.cache` at module level.

**Savings:** ~20 lines, 4 class keywords.

---

## 10. ProcessingService — 45-line wrapper

**File:** `src/services/processing_service.py`

**Problem:** Wraps `FileSystemRepository.read_file_async` with a size check and extension extraction. Called from exactly one use case.

**Replacement:** Inline the 20-line loop into `ProcessWorkspaceUseCase`. The check `os.path.getsize(path) > MAX_FILE_SIZE_MB` and `splitext` are stdlib calls, not service-worthy.

**Savings:** ~45 lines, 1 file.

---

## 11. DependencyService strategy-registry pattern — 142 lines across 2 files

**Files:** `src/services/dependency_service.py` (79), `src/services/strategies/dependency_strategies.py` (63)

**Problem:** Full Strategy pattern: ABC (`DependencyParserStrategy`), two implementations (`PythonDependencyParser`, `WebDependencyParser`), a registry (`self._parsers: Dict`), and a registration method (`register_strategy`). The actual logic is: if `.py` → `ast.parse`, if `.js/.ts/...` → regex.

**Replacement:** One function, one `dict[ext → callable]`:

```python
_PARSERS = {
    '.py': _parse_python,
    '.js': _parse_web, '.ts': _parse_web, ...
}
```

**Savings:** ~100 lines, 1 file.

---

## 12. ImportResolutionStrategy ABC + 5 subclasses — 111 lines

**File:** `src/services/strategies/import_strategies.py`

**Problem:** ABC, `StandardImportStrategy`, `FSDImportStrategy`, `AtomicDesignImportStrategy`, `DDDImportStrategy`, `MonorepoImportStrategy` — all with the same signature, used from a single `ImportResolutionService`. Each `is_match` is 1-5 lines.

**Replacement:** Single function with pattern matching or dict of lambdas. The monorepo strategy adds real complexity (30 lines), but even that doesn't warrant an ABC.

**Savings:** ~80 lines, 1 file.

---

## 13. `_THROTTLED_ACTIONS` in Dispatcher — speculative optimization

**File:** `src/actions/dispatcher.py:7-12`

**Problem:** Throttling for UI events implemented before profiling shows it's needed. 5 hardcoded action types, 33ms threshold. In a Qt app, widget updates are already coalesced by the event loop.

**Replacement:** Remove. Add when a profiler says "too many dispatches per frame."

**Savings:** ~8 lines.

---

## 14. Duplicate integration strategy factories

**Files:** `src/di_container.py:40-46`, `src/services/integration_service.py:22-29`

**Problem:** `_make_integration_strategy()` in DIContainer and `_create_default_strategy()` in IntegrationService do the exact same thing: detect OS and return the right strategy class.

**Replacement:** One factory. DIContainer's version is the one used; IntegrationService's version is only a fallback that never fires because DI always provides a strategy.

**Savings:** ~8 lines.

---

## 15. Duplicate CLI/GUI settings in AppSettings — 12 extra fields

**File:** `src/store/state.py`

**Problem:** Six pairs of settings where the CLI version shadows the GUI version:

- `minify` + `cli_minify`
- `remove_comments` + `cli_remove_comments`
- `remove_secrets` + `cli_remove_secrets`
- `include_tree` + `cli_include_tree`
- `skeleton_mode` + `cli_skeleton_mode`
- `use_gitignore` + `cli_use_gitignore`

CLI mode overrides are handled at the entry point (`cli_controller.py:164-166`). The dual fields add complexity to validation, persistence, and the state model.

**Replacement:** One setting per concept. CLI passes overrides as kwargs, not as separate settings fields.

**Savings:** ~12 lines, 6 fields removed from 2 dataclasses.

---

## 16. `pyperclip` dependency — already have PySide6

**File:** `src/services/output_service.py:17`, `requirements.txt`

**Problem:** `pyperclip` is installed for clipboard access. The app already depends on PySide6, which has `QApplication.clipboard().setText()` and `QApplication.clipboard().text()`.

**Replacement:** Use `QApplication.clipboard()` — 0 new deps, working in the GUI path already. For CLI mode, `pyperclip` is the correct fallback (no Qt event loop).

**Savings:** -1 dep (optional: import inside CLI path only).

---

## 17. `fpdf` dependency — PDF export is niche

**File:** `src/services/output_service.py:48`, `requirements.txt`

**Problem:** `fpdf` is a dependency for one feature (PDF export). The conversion is lossy (encodes to `latin-1` with replacement).

**Replacement:** Save as `.md` — text output is the same content. If PDF is genuinely needed, it can be an optional pip install with a `try/except ImportError`.

**Savings:** -1 dep.

---

## 18. `get_app_version` loops over 4 encodings — YAGNI

**File:** `src/utils/config.py:86-104`

**Problem:** The function iterates `("utf-8-sig", "utf-16", "utf-8", "cp1252")` looking for the right encoding for a file the project controls (`VERSION.txt`).

**Replacement:** `open(candidate, encoding='utf-8')`. The file is written by `bumpversion` — it's always UTF-8.

**Savings:** ~15 lines.

---

## 19. `create_default_logo` draws a logo dynamically — 15 lines

**File:** `main.py:38-54`

**Problem:** If `logo.png` doesn't exist, the app paints a "fake" QImage logo and saves it. This is a fallback for a bundle artifact that the installer/packager should ship.

**Replacement:** `assets/images/logo.png` exists in the repo. If frozen build doesn't include it, that's a packaging bug, not a runtime concern. Remove fallback, let it crash-loud if missing.

**Savings:** ~15 lines.

---

## 20. `AsyncRuntime` — 46 lines of boilerplate

**File:** `src/utils/async_runtime.py`

**Problem:** Custom thread + event loop management class. The app needs to run coroutines from Qt's synchronous event loop.

**Replacement:** Use `qasync` if already available (not currently a dep). Or, more practically, the class is fine — just note it could be replaced with `asyncio.run()` for CLI-only calls and `qasync` for GUI mode. Flagged for awareness, not deletion — the need is real.

**Savings:** 0 now, -46 if `qasync` is adopted.

---

## 21. Empty `__init__.py` files — 10 clutter files

**Locations:** `src/store/__init__.py`, `src/actions/__init__.py`, `src/use_cases/__init__.py`, `src/controllers/__init__.py`, `src/services/__init__.py`, `src/services/strategies/__init__.py`, `src/ui/__init__.py`, `src/ui/components/__init__.py`, `src/utils/__init__.py`, `tests/__init__.py`, `src/data/__init__.py`

**Problem:** 11 empty `__init__.py` files. Python 3.3+ supports namespace packages without `__init__.py`. The files serve no purpose.

**Replacement:** Delete them all. They're resolved as namespace packages implicitly.

**Savings:** 0 lines of logic, 11 files removed.

---

## 22. `PRESETS` with "C++ / Embedded" — unused

**File:** `src/utils/config.py:21-25`

**Problem:** A preset for C++ / Embedded that no UI element references and no CLI code selects. The app has no C++ detection.

**Replacement:** Delete. Add when a C++ dev profile is actually needed.

**Savings:** ~5 lines.

---

## 23. `_watcher_loop` polling thread — manual `os.walk`-based file watcher

**File:** `src/services/file_service.py:113-140`

**Problem:** A polling file watcher that does `os.walk` every `interval` seconds and compares `mtime`. This is what `watchdog` or inotify/kqueue do natively. But since adding `watchdog` would be a new dep, this is borderline.

**Flag:** Consider `watchdog` if the watcher becomes a bottleneck. For now, it's functional and zero-dep. Not flagged as delete.

---

## Summary

| Category | Count | Lines affected |
|----------|-------|---------------|
| Delete (YAGNI / dead) | 10 | ~720 |
| Shrink (simplify) | 5 | ~250 |
| Stdlib (use platform) | 2 | ~10 |
| Native (use Qt) | 1 | ~5 |
| Dep removal | 2 | -2 deps |

**net: -1450 lines, -2 deps possible.**

*Leanest wins: drop pyperclip + fpdf, flatten strategies, delete the redundant service wrappers, and replace Redux with Qt signals.*
