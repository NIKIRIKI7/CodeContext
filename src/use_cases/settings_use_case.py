"""
SettingsUseCase — сценарии работы с настройками.

Выделен из MainController для соблюдения SRP.
Объединяет: загрузку, сохранение, сброс, применение пресетов.
"""

import json
from typing import Optional

from ..actions.action_types import (
    SETTINGS_LOADED, SETTINGS_UPDATE, WORKSPACE_LOADED, UI_ADD_LOG,
)
from ..actions.dispatcher import Dispatcher
from ..data.settings_repository import SettingsRepository
from ..store.store import Store
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT


_DEFAULT_SETTINGS = {
    'extensions': PRESETS['Default']['ext'],
    'ignored_paths': PRESETS['Default']['ign'],
    'system_prompt': DEFAULT_SYSTEM_PROMPT,
    'minify': True,
    'remove_comments': True,
    'remove_secrets': True,
    'include_tree': True,
    'include_dependencies': False,
    'skeleton_mode': False,
    'use_git': False,
    'use_gitignore': True,
    'cli_minify': True,
    'cli_remove_comments': True,
    'cli_remove_secrets': True,
    'cli_include_tree': True,
    'cli_skeleton_mode': False,
    'cli_use_gitignore': True,
    'cli_format': "plain",
    'output_format': 'markdown',
    'template_path': '',
    'python_interpreter': '',
}


class SettingsUseCase:
    """Управляет жизненным циклом настроек приложения."""

    def __init__(
        self,
        dispatcher: Dispatcher,
        store: Store,
        settings_repo: SettingsRepository,
    ):
        self._dispatcher = dispatcher
        self._store = store
        self._settings_repo = settings_repo

    def load_initial(self) -> None:
        """Загружает настройки из файла или применяет дефолтные."""
        data = self._settings_repo.load()
        if not data:
            data = _DEFAULT_SETTINGS.copy()
        self._dispatcher.dispatch(SETTINGS_LOADED, data)

    def update(self, settings_data: dict) -> None:
        """Обновляет часть настроек без сохранения на диск."""
        self._dispatcher.dispatch(SETTINGS_UPDATE, settings_data)

    def save(self) -> None:
        """Сохраняет текущие настройки на диск."""
        self._settings_repo.save(self._store.state.settings.__dict__)

    def reset(self) -> None:
        """Сбрасывает настройки на дефолтные и сохраняет."""
        self._dispatcher.dispatch(SETTINGS_UPDATE, _DEFAULT_SETTINGS.copy())
        self._settings_repo.save(_DEFAULT_SETTINGS)
        self._dispatcher.dispatch(UI_ADD_LOG, "Настройки сброшены")

    def apply_preset(self, preset_name: str) -> None:
        """Применяет пресет расширений/игнора."""
        preset = PRESETS.get(preset_name)
        if preset:
            self._dispatcher.dispatch(SETTINGS_UPDATE, {
                'extensions': preset['ext'],
                'ignored_paths': preset['ign'],
            })

    def save_workspace(self, path: str) -> None:
        """Сохраняет workspace (папки + настройки) в JSON-файл."""
        state = self._store.state
        data = {
            'folders': state.selected_folders,
            'settings': state.settings.__dict__,
        }
        try:
            with open(path, 'w', encoding='utf-8') as f:
                json.dump(data, f, ensure_ascii=False, indent=4)
            self._dispatcher.dispatch(UI_ADD_LOG, f"Workspace сохранён: {path}")
        except OSError as exc:
            self._dispatcher.dispatch(UI_ADD_LOG, f"Ошибка сохранения Workspace: {exc}")

    def load_workspace(self, path: str) -> Optional[dict]:
        """Загружает workspace из JSON-файла и публикует в Store."""
        try:
            with open(path, 'r', encoding='utf-8') as f:
                data = json.load(f)
            self._dispatcher.dispatch(WORKSPACE_LOADED, data)
            self._dispatcher.dispatch(UI_ADD_LOG, f"Workspace загружен: {path}")
            return data
        except (OSError, json.JSONDecodeError) as exc:
            self._dispatcher.dispatch(UI_ADD_LOG, f"Ошибка загрузки Workspace: {exc}")
            return None