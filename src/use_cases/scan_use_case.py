import os
import json

from ..actions.action_types import (
    UI_SET_LOADING, UI_UPDATE_STATUS, UI_ADD_LOG,
    SCAN_SUCCESS, SCAN_FAILURE,
)
from ..actions.dispatcher import Dispatcher
from ..data.file_system_repository import FileSystemRepository
from ..services.file_service import FileService
from ..store.state import AppState
from src.i18n import tr
from ..utils.logger import app_logger
from ..utils.config import get_app_data_dir


class ScanWorkspaceUseCase:
    """Сканирует выбранные папки и публикует результат в Store."""

    def __init__(self, dispatcher: Dispatcher, file_service: FileService, fs_repo: FileSystemRepository):
        self._dispatcher = dispatcher
        self._file_service = file_service
        self._fs_repo = fs_repo

    def _load_cache(self) -> dict:
        cache_path = os.path.join(get_app_data_dir(), "scan_cache.json")
        try:
            if os.path.exists(cache_path):
                with open(cache_path, 'r', encoding='utf-8') as f:
                    return json.load(f)
        except Exception:
            pass
        return {}

    def _save_cache(self, cache: dict):
        cache_path = os.path.join(get_app_data_dir(), "scan_cache.json")
        try:
            with open(cache_path, 'w', encoding='utf-8') as f:
                json.dump(cache, f)
        except Exception:
            pass

    async def execute(self, state: AppState) -> None:
        self._dispatcher.dispatch(UI_SET_LOADING, True)
        self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': tr("scan_use_case.scanning"), 'progress': 0.0})
        self._dispatcher.dispatch(UI_ADD_LOG, tr("scan_use_case.scan_started"))

        try:
            file_paths = await self._file_service.scan_folders_async(
                paths=state.selected_folders,
                extensions_str=state.settings.extensions,
                ignored_str=state.settings.ignored_paths,
                use_git=state.settings.use_git,
                use_gitignore=state.settings.use_gitignore,
            )

            if not file_paths:
                self._dispatcher.dispatch(SCAN_FAILURE, tr("scan_use_case.no_files"))
                self._dispatcher.dispatch(UI_ADD_LOG, tr("scan_use_case.no_files"))
            else:
                self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': tr("scan_use_case.analyzing"), 'progress': 0.5})

                git_statuses = {}
                for folder in state.selected_folders:
                    folder_status = await self._fs_repo.get_git_status_async(folder)
                    git_statuses.update(folder_status)

                token_cache = self._load_cache()
                new_cache = {}
                metadata = {}

                for path in file_paths:
                    try:
                        stat = os.stat(path)
                        cache_key = f"{path}_{stat.st_size}_{stat.st_mtime}"
                        if cache_key in token_cache:
                            tokens = token_cache[cache_key]
                        else:
                            tokens = stat.st_size // 4
                        new_cache[cache_key] = tokens
                    except OSError:
                        app_logger.warning(f"[Scan] Cannot get file size: {path}")
                        tokens = 0

                    status = git_statuses.get(path, "")
                    category = self._determine_file_category(path, tokens)

                    metadata[path] = {
                        "tokens": tokens,
                        "git_status": status,
                        "category": category
                    }

                self._save_cache(new_cache)

                self._dispatcher.dispatch(SCAN_SUCCESS, {'paths': file_paths, 'metadata': metadata})
                self._dispatcher.dispatch(UI_ADD_LOG, tr("store.status.files_found", count=len(file_paths)))

        except Exception as exc:
            self._dispatcher.dispatch(SCAN_FAILURE, str(exc))
            self._dispatcher.dispatch(UI_ADD_LOG, tr("store.status.scan_error", error=exc))
        finally:
            self._dispatcher.dispatch(UI_SET_LOADING, False)
            self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': tr("scan_use_case.scan_complete"), 'progress': 0.0})

    @staticmethod
    def _determine_file_category(path: str, tokens: int) -> str:
        """Определяет категорию файла на основе пути (зависимости) и размера."""
        normalized_path = path.replace('\\', '/')
        path_parts = normalized_path.split('/')

        # Список директорий, которые считаются внешними зависимостями или билдами
        dependency_dirs = {
            'node_modules', 'venv', '.venv', 'env', '.env',
            'dist', 'build', '__pycache__', 'target', 'out', 'vendor'
        }

        if any(part in dependency_dirs for part in path_parts):
            return "DEPENDENCY"

        if tokens > 50000:
            return "HUGE"
        elif tokens > 25000:
            return "HEAVY"
        elif tokens > 5000:
            return "MEDIUM"

        return "LIGHT"