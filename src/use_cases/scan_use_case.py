import os

from src.i18n import tr
from ..services.file_service import FileService, get_git_status_async
from ..store.state import AppState


async def scan_workspace(state: AppState) -> None:
    file_service = FileService()

    state.is_loading = True
    state.status_message = tr("scan_use_case.scanning")
    state.notify()

    try:
        file_paths = await file_service.scan_folders_async(
            paths=state.selected_folders,
            extensions_str=state.settings.extensions,
            ignored_str=state.settings.ignored_paths,
            use_git=state.settings.use_git,
            use_gitignore=state.settings.use_gitignore,
            git_base=state.settings.git_base,
        )

        if not file_paths:
            state.scanned_files_paths = []
            state.status_message = tr("scan_use_case.no_files")
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
                    "category": _determine_file_category(path, tokens)
                }

            state.scanned_files_paths = file_paths
            state.scanned_file_metadata = metadata
            state.manual_exclusions = set()
            _recalculate_tokens(state)
            state.status_message = tr("store.status.files_found", count=len(file_paths))

    except Exception as exc:
        state.scanned_files_paths = []
        state.status_message = tr("store.status.scan_error")
    finally:
        state.is_loading = False
        state.notify()


def _recalculate_tokens(state: AppState):
    state.selected_tokens = sum(
        meta.get("tokens", 0) for path, meta in state.scanned_file_metadata.items()
        if path not in state.manual_exclusions
    )


def _determine_file_category(path: str, tokens: int) -> str:
    parts = path.replace('\\', '/').split('/')
    if any(p in {'node_modules', 'venv', '.venv', 'env', '.env', 'dist', 'build', '__pycache__', 'target', 'out', 'vendor'} for p in parts):
        return "DEPENDENCY"
    if tokens > 50000: return "HUGE"
    if tokens > 25000: return "HEAVY"
    if tokens > 5000: return "MEDIUM"
    return "LIGHT"
