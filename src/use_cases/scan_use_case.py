import os
import asyncio
from ..actions.action_types import (
    UI_SET_LOADING, UI_UPDATE_STATUS, UI_ADD_LOG,
    SCAN_SUCCESS, SCAN_FAILURE,
)
from ..actions.dispatcher import Dispatcher
from ..services.file_service import FileService
from ..data.file_system_repository import FileSystemRepository
from ..store.state import AppState

class ScanWorkspaceUseCase:
    """Сканирует выбранные папки и публикует результат в Store."""
    def __init__(self, dispatcher: Dispatcher, file_service: FileService, fs_repo: FileSystemRepository):
        self._dispatcher = dispatcher
        self._file_service = file_service
        self._fs_repo = fs_repo

    async def execute(self, state: AppState) -> None:
        self._dispatcher.dispatch(UI_SET_LOADING, True)
        self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование...", 'progress': 0.0})
        self._dispatcher.dispatch(UI_ADD_LOG, "Начало сканирования...")

        try:
            file_paths = await self._file_service.scan_folders_async(
                paths=state.selected_folders,
                extensions_str=state.settings.extensions,
                ignored_str=state.settings.ignored_paths,
                use_git=state.settings.use_git,
                use_gitignore=state.settings.use_gitignore,
            )

            if not file_paths:
                self._dispatcher.dispatch(SCAN_FAILURE, "Файлы не найдены")
                self._dispatcher.dispatch(UI_ADD_LOG, "⚠️ Файлы не найдены")
            else:
                self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Анализ токенов и Git...", 'progress': 0.5})

                git_statuses = {}
                for folder in state.selected_folders:
                    folder_status = await self._fs_repo.get_git_status_async(folder)
                    git_statuses.update(folder_status)

                metadata = {}
                for path in file_paths:
                    try:
                        tokens = os.path.getsize(path) // 4
                    except OSError:
                        tokens = 0

                    status = git_statuses.get(path, "")
                    metadata[path] = {"tokens": tokens, "git_status": status}

                self._dispatcher.dispatch(SCAN_SUCCESS, {'paths': file_paths, 'metadata': metadata})
                self._dispatcher.dispatch(UI_ADD_LOG, f"Найдено файлов: {len(file_paths)}")

        except Exception as exc:
            self._dispatcher.dispatch(SCAN_FAILURE, str(exc))
            self._dispatcher.dispatch(UI_ADD_LOG, f"Ошибка сканирования: {exc}")
        finally:
            self._dispatcher.dispatch(UI_SET_LOADING, False)
            self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование завершено", 'progress': 0.0})