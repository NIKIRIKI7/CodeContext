"""
Store — центральное хранилище состояния (Redux-like).

Изменения:
- Редьюсер заменён реестром обработчиков (Dict[str, Callable]).
  Добавление нового action = регистрация нового метода (OCP).
- _handle_* методы изолированы — каждый отвечает за один action (SRP).
"""

import copy
from typing import List, Callable, Any, Dict
from .state import AppState, AppSettings
from ..actions.action_types import *


class Store:
    """Центральное хранилище состояния."""

    def __init__(self):
        self._state = AppState()
        self._listeners: List[Callable[[AppState], None]] = []
        self._handlers: Dict[str, Callable[[Any], None]] = {}
        self._register_handlers()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    @property
    def state(self) -> AppState:
        """Всегда возвращает глубокую копию — защита от внешних мутаций."""
        return copy.deepcopy(self._state)

    def subscribe(self, listener: Callable[[AppState], None]) -> Callable:
        """Подписка на изменения. Возвращает функцию отписки."""
        self._listeners.append(listener)
        return lambda: self._listeners.remove(listener)

    def dispatch(self, action_type: str, payload: Any = None):
        """Изменение состояния через зарегистрированный обработчик."""
        handler = self._handlers.get(action_type)
        if handler:
            handler(payload)
        # Уведомляем подписчиков даже если action не найден
        # (позволяет логировать неизвестные action'ы на уровне UI)
        self._notify()

    # ------------------------------------------------------------------
    # Registration (OCP: новый action → новый метод, без изменения dispatch)
    # ------------------------------------------------------------------

    def _register_handlers(self):
        self._handlers.update({
            UI_SET_LOADING:       self._handle_set_loading,
            UI_UPDATE_STATUS:     self._handle_update_status,
            UI_ADD_LOG:           self._handle_add_log,
            UI_SHOW_PREVIEW:      self._handle_show_preview,
            UI_CLOSE_PREVIEW:     self._handle_close_preview,
            SETTINGS_LOADED:      self._handle_settings_loaded,
            SETTINGS_UPDATE:      self._handle_settings_update,

            WORKSPACE_LOADED:     self._handle_workspace_loaded,

            FOLDER_ADD:           self._handle_folder_add,
            FOLDER_REMOVE:        self._handle_folder_remove,
            FOLDER_UPDATE:        self._handle_folder_update,
            FOLDER_CLEAR:         self._handle_folder_clear,

            GITHUB_CLONE_SUCCESS: self._handle_github_success,
            GITHUB_CLONE_FAILURE: self._handle_github_failure,

            SCAN_SUCCESS:         self._handle_scan_success,
            SCAN_FAILURE:         self._handle_scan_failure,

            EXCLUSION_ADD:        self._handle_exclusion_add,
            EXCLUSION_REMOVE:     self._handle_exclusion_remove,
            EXCLUSION_CLEAR:      lambda _: self._state.__setattr__('manual_exclusions', set()),

            PROCESSING_SUCCESS:   lambda p: self._state.__setattr__('processed_files', p),

            FORMATTING_SUCCESS:   self._handle_formatting_success,

            WORKFLOW_STARTED:     self._handle_update_status,
            WORKFLOW_PROGRESS:    self._handle_update_status,
            WORKFLOW_FINISHED:    lambda _: self._handle_update_status({'message': 'Готово', 'progress': 1.0}),
            WORKFLOW_ERROR:       self._handle_workflow_error,
        })

    # ------------------------------------------------------------------
    # Handlers (SRP: один handler — одно действие)
    # ------------------------------------------------------------------

    def _handle_set_loading(self, payload: bool):
        self._state.is_loading = bool(payload)

    def _handle_update_status(self, payload: dict):
        if not isinstance(payload, dict):
            return
        self._state.status_message = payload.get('message', self._state.status_message)
        self._state.progress = payload.get('progress', self._state.progress)

    def _handle_add_log(self, payload):
        self._state.logs.append(str(payload))

    def _handle_show_preview(self, payload: str):
        self._state.preview_text = payload
        self._state.show_preview = True

    def _handle_close_preview(self, _):
        self._state.show_preview = False
        self._state.preview_text = ""

    def _handle_settings_loaded(self, payload: dict):
        self._state.settings = AppSettings(**payload)

    def _handle_settings_update(self, payload: dict):
        current = self._state.settings.__dict__.copy()
        current.update(payload)
        self._state.settings = AppSettings(**current)

    def _handle_workspace_loaded(self, payload: dict):
        self._state.selected_folders = payload.get('folders', [])
        current = self._state.settings.__dict__.copy()
        current.update(payload.get('settings', {}))
        self._state.settings = AppSettings(**current)
        self._state.temp_folders = []

    def _handle_folder_add(self, path: str):
        if path not in self._state.selected_folders:
            self._state.selected_folders.append(path)

    def _handle_folder_remove(self, path: str):
        if path in self._state.selected_folders:
            self._state.selected_folders.remove(path)
        if path in self._state.temp_folders:
            self._state.temp_folders.remove(path)

    def _handle_folder_update(self, payload: dict):
        old, new = payload['old'], payload['new']
        for collection in (self._state.selected_folders, self._state.temp_folders):
            if old in collection:
                collection[collection.index(old)] = new

    def _handle_folder_clear(self, _):
        self._state.selected_folders = []
        self._state.temp_folders = []
        self._state.scanned_files_paths = []
        self._state.processed_files = []
        self._state.manual_exclusions = set()
        self._state.final_output_text = ""
        self._state.total_tokens = 0
        self._state.status_message = "Рабочая область очищена"
        self._state.progress = 0.0
        self._state.logs = []

    def _handle_github_success(self, path: str):
        if path not in self._state.selected_folders:
            self._state.selected_folders.append(path)
        if path not in self._state.temp_folders:
            self._state.temp_folders.append(path)
        self._state.status_message = "Репозиторий загружен"
        self._state.logs.append(f"GitHub Repo Cloned: {path}")

    def _handle_github_failure(self, error: str):
        self._state.status_message = "Ошибка клонирования"
        self._state.logs.append(f"GitHub Error: {error}")

    def _handle_scan_success(self, paths: list):
        self._state.scanned_files_paths = paths
        self._state.manual_exclusions = set()
        self._state.status_message = f"Найдено файлов: {len(paths)}"

    def _handle_scan_failure(self, error: str):
        self._state.scanned_files_paths = []
        self._state.status_message = "Ошибка сканирования"
        self._state.logs.append(f"Error: {error}")

    def _handle_exclusion_add(self, path: str):
        self._state.manual_exclusions.add(path)

    def _handle_exclusion_remove(self, path: str):
        self._state.manual_exclusions.discard(path)

    def _handle_formatting_success(self, payload: dict):
        self._state.final_output_text = payload['text']
        self._state.total_tokens = payload['tokens']
        self._state.status_message = "Готово"
        self._state.progress = 1.0
        self._state.is_loading = False

    def _handle_workflow_error(self, error: str):
        self._state.status_message = f"Ошибка: {error}"
        self._state.is_loading = False
        self._state.logs.append(f"CRITICAL ERROR: {error}")

    # ------------------------------------------------------------------
    # Notifications
    # ------------------------------------------------------------------

    def _notify(self):
        snapshot = self.state
        for listener in self._listeners:
            listener(snapshot)