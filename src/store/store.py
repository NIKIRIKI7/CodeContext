import copy
from typing import List, Callable, Any
from .state import AppState, AppSettings
from ..actions.action_types import *


class Store:
    """Центральное хранилище состояния (Redux-like)"""

    def __init__(self):
        self._state = AppState()
        self._listeners: List[Callable[[AppState], None]] = []

    @property
    def state(self) -> AppState:
        """Возвращает копию состояния (для чтения)"""
        return copy.deepcopy(self._state)

    def subscribe(self, listener: Callable[[AppState], None]):
        """Подписка на изменения состояния"""
        self._listeners.append(listener)
        return lambda: self._listeners.remove(listener)

    def _notify(self):
        """Уведомление подписчиков"""
        state_snapshot = self.state
        for listener in self._listeners:
            listener(state_snapshot)

    def dispatch(self, action_type: str, payload: Any = None):
        """Изменение состояния через редьюсер"""
        self._reducer(action_type, payload)
        self._notify()

    def _reducer(self, action_type: str, payload: Any):
        """Чистая функция изменения состояния"""

        if action_type == UI_SET_LOADING:
            self._state.is_loading = payload

        elif action_type == UI_UPDATE_STATUS:
            self._state.status_message = payload.get('message', '')
            self._state.progress = payload.get('progress', 0.0)

        elif action_type == UI_ADD_LOG:
            self._state.logs.append(str(payload))

        elif action_type == SETTINGS_LOADED:
            self._state.settings = AppSettings(**payload)

        elif action_type == SETTINGS_UPDATE:
            current_dict = self._state.settings.__dict__
            current_dict.update(payload)
            self._state.settings = AppSettings(**current_dict)

        elif action_type == FOLDER_ADD:
            if payload not in self._state.selected_folders:
                self._state.selected_folders.append(payload)

        elif action_type == FOLDER_REMOVE:
            # <--- NEW: Удаление конкретной папки
            path_to_remove = payload
            if path_to_remove in self._state.selected_folders:
                self._state.selected_folders.remove(path_to_remove)
            # Если это была временная папка (GitHub), удаляем и оттуда (физическое удаление оставим на clear/exit)
            if path_to_remove in self._state.temp_folders:
                self._state.temp_folders.remove(path_to_remove)

        elif action_type == FOLDER_UPDATE:
            # <--- NEW: Обновление пути (редактирование)
            old_path = payload['old']
            new_path = payload['new']
            if old_path in self._state.selected_folders:
                idx = self._state.selected_folders.index(old_path)
                self._state.selected_folders[idx] = new_path

            # Также обновляем в temp_folders если нужно
            if old_path in self._state.temp_folders:
                idx = self._state.temp_folders.index(old_path)
                self._state.temp_folders[idx] = new_path

        elif action_type == GITHUB_CLONE_SUCCESS:
            path = payload
            if path not in self._state.selected_folders:
                self._state.selected_folders.append(path)
            if path not in self._state.temp_folders:
                self._state.temp_folders.append(path)
            self._state.status_message = "Репозиторий загружен"
            self._state.logs.append(f"GitHub Repo Cloned: {path}")

        elif action_type == GITHUB_CLONE_FAILURE:
            self._state.status_message = "Ошибка клонирования"
            self._state.logs.append(f"GitHub Error: {payload}")

        elif action_type == FOLDER_CLEAR:
            self._state.selected_folders = []
            self._state.temp_folders = []
            self._state.scanned_files_paths = []
            self._state.processed_files = []
            self._state.final_output_text = ""
            self._state.total_tokens = 0
            self._state.status_message = "Рабочая область очищена"
            self._state.progress = 0.0
            self._state.logs = []

        elif action_type == SCAN_SUCCESS:
            self._state.scanned_files_paths = payload
            self._state.status_message = f"Найдено файлов: {len(payload)}"

        elif action_type == SCAN_FAILURE:
            self._state.scanned_files_paths = []
            self._state.status_message = "Ошибка сканирования"
            self._state.logs.append(f"Error: {payload}")

        elif action_type == PROCESSING_SUCCESS:
            self._state.processed_files = payload

        elif action_type == FORMATTING_SUCCESS:
            self._state.final_output_text = payload['text']
            self._state.total_tokens = payload['tokens']
            self._state.status_message = "Готово"
            self._state.progress = 1.0
            self._state.is_loading = False