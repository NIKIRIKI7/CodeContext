import os

from ..data.file_system_repository import FileSystemRepository
from ..services.file_service import FileService
from ..store.state import AppState
from src.i18n import tr
from ..utils.logger import app_logger


class ScanWorkspaceUseCase:
    def __init__(self, state: AppState, file_service: FileService, fs_repo: FileSystemRepository):
        self.state = state
        self._file_service = file_service
        self._fs_repo = fs_repo

    async def execute(self, state: AppState) -> None:
        self.state.is_loading = True
        self.state.status_message = tr("scan_use_case.scanning")
        self.state.progress = 0.0
        self.state.notify()
        self.state.add_log(tr("scan_use_case.scan_started"))

        try:
            file_paths = await self._file_service.scan_folders_async(
                paths=state.selected_folders,
                extensions_str=state.settings.extensions,
                ignored_str=state.settings.ignored_paths,
                use_git=state.settings.use_git,
                use_gitignore=state.settings.use_gitignore,
                git_base=state.settings.git_base,
            )

            if not file_paths:
                self.state.scanned_files_paths = []
                self.state.status_message = tr("scan_use_case.no_files")
                self.state.add_log(tr("scan_use_case.no_files"))
            else:
                self.state.status_message = tr("scan_use_case.analyzing")
                self.state.progress = 0.5
                self.state.notify()

                git_statuses = {}
                for folder in state.selected_folders:
                    folder_status = await self._fs_repo.get_git_status_async(folder)
                    git_statuses.update(folder_status)

                metadata = {}

                for path in file_paths:
                    try:
                        stat = os.stat(path)
                        tokens = stat.st_size // 4
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

                self.state.scanned_files_paths = file_paths
                self.state.scanned_file_metadata = metadata
                self.state.manual_exclusions = set()
                self._recalculate_tokens()
                self.state.status_message = tr("store.status.files_found", count=len(file_paths))
                self.state.notify()
                self.state.add_log(tr("store.status.files_found", count=len(file_paths)))

        except Exception as exc:
            self.state.scanned_files_paths = []
            self.state.status_message = tr("store.status.scan_error")
            self.state.add_log(tr("store.status.scan_error_log", error=str(exc)))
            self.state.notify()
        finally:
            self.state.is_loading = False
            self.state.status_message = tr("scan_use_case.scan_complete")
            self.state.progress = 0.0
            self.state.notify()

    def _recalculate_tokens(self):
        total = 0
        for path, meta in self.state.scanned_file_metadata.items():
            if path not in self.state.manual_exclusions:
                total += meta.get("tokens", 0)
        self.state.selected_tokens = total

    @staticmethod
    def _determine_file_category(path: str, tokens: int) -> str:
        normalized_path = path.replace('\\', '/')
        path_parts = normalized_path.split('/')
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
