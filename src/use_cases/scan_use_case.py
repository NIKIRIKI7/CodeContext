import os

from src.i18n import tr
from ..services.file_service import FileService, get_git_status_async
from ..store.state import AppState


class ScanWorkspaceUseCase:
    def __init__(self, state: AppState, file_service: FileService):
        self.state = state
        self._file_service = file_service

    async def execute(self, state: AppState) -> None:
        self.state.is_loading = True
        self.state.status_message = tr("scan_use_case.scanning")
        self.state.notify()
        
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
            else:
                git_statuses = {}
                for folder in state.selected_folders:
                    git_statuses.update(await get_git_status_async(folder))
                
                metadata = {}
                for path in file_paths:
                    try:
                        tokens = os.stat(path).st_size // 4
                    except OSError:
                        tokens = 0
                    metadata[path] = {
                        "tokens": tokens,
                        "git_status": git_statuses.get(path, ""),
                        "category": self._determine_file_category(path, tokens)
                    }
                
                self.state.scanned_files_paths = file_paths
                self.state.scanned_file_metadata = metadata
                self.state.manual_exclusions = set()
                self._recalculate_tokens()
                self.state.status_message = tr("store.status.files_found", count=len(file_paths))
                
        except Exception as exc:
            self.state.scanned_files_paths = []
            self.state.status_message = tr("store.status.scan_error")
        finally:
            self.state.is_loading = False
            self.state.notify()

    def _recalculate_tokens(self):
        self.state.selected_tokens = sum(
            meta.get("tokens", 0) for path, meta in self.state.scanned_file_metadata.items()
            if path not in self.state.manual_exclusions
        )

    @staticmethod
    def _determine_file_category(path: str, tokens: int) -> str:
        parts = path.replace('\\', '/').split('/')
        if any(p in {'node_modules', 'venv', '.venv', 'env', '.env', 'dist', 'build', '__pycache__', 'target', 'out', 'vendor'} for p in parts):
            return "DEPENDENCY"
        if tokens > 50000: return "HUGE"
        if tokens > 25000: return "HEAVY"
        if tokens > 5000: return "MEDIUM"
        return "LIGHT"
