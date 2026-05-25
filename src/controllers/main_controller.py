import os
from typing import Optional, Tuple, List
from PySide6.QtCore import QTimer

from ..actions.action_types import (
    FOLDER_ADD, FOLDER_REMOVE, FOLDER_UPDATE, FOLDER_CLEAR,
    EXCLUSION_ADD, EXCLUSION_REMOVE, EXCLUSION_CLEAR,
    UI_ADD_LOG, UI_CLOSE_PREVIEW, UI_SHOW_TOUR, UI_CLOSE_TOUR
)
from ..actions.dispatcher import Dispatcher
from ..store.store import Store
from ..store.state import ProcessedFile  # Добавлен импорт для режима "none"
from ..use_cases.scan_use_case import ScanWorkspaceUseCase
from ..use_cases.process_use_case import ProcessWorkspaceUseCase
from ..use_cases.github_use_case import GitHubUseCase
from ..use_cases.settings_use_case import SettingsUseCase
from ..use_cases.patch_use_case import PatchUseCase
from ..utils.async_runtime import AsyncRuntime

from ..services.integration_service import IntegrationService
from ..services.tour_service import TourService
from ..services.llm_checker_service import LlmCheckerService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..data.file_system_repository import FileSystemRepository

class MainController:
    def __init__(
        self,
        store: Store,
        dispatcher: Dispatcher,
        scan_use_case: ScanWorkspaceUseCase,
        process_use_case: ProcessWorkspaceUseCase,
        github_use_case: GitHubUseCase,
        settings_use_case: SettingsUseCase,
        patch_use_case: PatchUseCase,
        integration_service: IntegrationService,
        fs_repo: FileSystemRepository,
        tour_service: TourService,
        llm_checker: LlmCheckerService,
        format_service: FormattingService,
        output_service: OutputService,
    ):
        self._store = store
        self._dispatcher = dispatcher
        self._scan_uc = scan_use_case
        self._process_uc = process_use_case
        self._github_uc = github_use_case
        self._settings_uc = settings_use_case
        self._patch_uc = patch_use_case
        self._integration = integration_service
        self._fs_repo = fs_repo
        self._tour_service = tour_service
        self._llm_checker = llm_checker
        self._format_service = format_service
        self._output_service = output_service

    def load_initial_settings(self):
        self._settings_uc.load_initial()

    def update_settings(self, data: dict):
        self._settings_uc.update(data)

    def save_settings(self):
        self._settings_uc.save()

    def reset_settings(self):
        self._settings_uc.reset()

    def apply_preset(self, preset_name: str):
        self._settings_uc.apply_preset(preset_name)

    def save_workspace(self, path: str):
        self._settings_uc.save_workspace(path)

    def load_workspace(self, path: str):
        self._settings_uc.load_workspace(path)
        self.scan_only()

    def add_folder(self, path: str):
        clean = self._normalize_path(path)
        if clean and os.path.exists(clean):
            self._dispatcher.dispatch(FOLDER_ADD, clean)
            recent = list(self._store.state.settings.recent_workspaces)
            if clean in recent:
                recent.remove(clean)
            recent.insert(0, clean)
            recent = recent[:6]
            self._settings_uc.update({'recent_workspaces': recent})
            self._settings_uc.save()

    def remove_folder(self, path: str):
        self._dispatcher.dispatch(FOLDER_REMOVE, path)

    def edit_folder(self, old_path: str, new_path: str):
        clean = self._normalize_path(new_path)
        if clean and clean != old_path:
            self._dispatcher.dispatch(FOLDER_UPDATE, {'old': old_path, 'new': clean})

    def clear_folders(self):
        temp = self._store.state.temp_folders
        if temp:
            self._dispatcher.dispatch(UI_ADD_LOG, "Очистка временных файлов...")
            AsyncRuntime.run_coroutine(self._clear_temp_async(temp))
        else:
            self._dispatcher.dispatch(FOLDER_CLEAR, None)

    async def _clear_temp_async(self, folders):
        for folder in folders:
            await self._fs_repo.delete_directory_async(folder)
        self._dispatcher.dispatch(FOLDER_CLEAR, None)

    def scan_only(self):
        if not self._store.state.selected_folders:
            self._dispatcher.dispatch(UI_ADD_LOG, "⚠️ Выберите папки для сканирования")
            return
        state = self._store.state
        AsyncRuntime.run_coroutine(self._scan_uc.execute(state))

    def start_processing(self, target: str, save_path: Optional[str] = None) -> Tuple[bool, str]:
        if not self._store.state.selected_folders:
            return False, "Выберите папки или URL для сканирования"
        state = self._store.state
        if not state.scanned_files_paths:
            AsyncRuntime.run_coroutine(self._scan_then_process(target, save_path))
        else:
            AsyncRuntime.run_coroutine(self._process_uc.execute(state, target, save_path))
        return True, ""

    async def _scan_then_process(self, target: str, save_path: Optional[str]):
        state = self._store.state
        await self._scan_uc.execute(state)
        state = self._store.state
        if state.scanned_files_paths:
            await self._process_uc.execute(state, target, save_path)

    def toggle_file_exclusion(self, path: str, is_included: bool):
        action = EXCLUSION_REMOVE if is_included else EXCLUSION_ADD
        self._dispatcher.dispatch(action, path)

    def copy_file_with_dependencies(self, target_file: str, mode: str):
        state = self._store.state
        AsyncRuntime.run_coroutine(self._copy_deps_async(target_file, mode, state))

    async def _copy_deps_async(self, target_file: str, mode: str, state):
        if mode == 'none':
            content = await self._fs_repo.read_file_async(target_file)
            if content is not None:
                pf = ProcessedFile(path=target_file, content=content, tokens=len(content)//4)
                text = self._format_service.format_output(
                    files=[pf],
                    fmt=state.settings.output_format,
                    include_tree=False,
                    system_prompt=""
                )
                self.copy_to_clipboard(text)
            else:
                self._dispatcher.dispatch(UI_ADD_LOG, f"⚠️ Не удалось прочитать файл: {target_file}")
        else:
            self._dispatcher.dispatch(UI_ADD_LOG, f"copy_with_deps (mode: {mode}): используйте CopyWithDepsUseCase")

    def add_github_repo(self, url: str, dest_path: str = ""):
        AsyncRuntime.run_coroutine(self._github_uc.execute(url, dest_path))

    def install_context_menu(self) -> Tuple[bool, str]:
        python_path = self._store.state.settings.python_interpreter
        return self._integration.install_context_menu(python_path)

    def remove_context_menu(self) -> Tuple[bool, str]:
        return self._integration.remove_context_menu()

    def close_preview(self):
        self._dispatcher.dispatch(UI_CLOSE_PREVIEW, None)

    def prepare_llm_patch(self, json_str: str) -> list:
        folders = self._store.state.selected_folders
        return self._patch_uc.prepare_json_patch(json_str, folders)

    def verify_patch_with_llm(self, patch: dict, callback):
        async def task():
            verdict = await self._llm_checker.check_patch(
                patch.get('original_content', ''),
                patch.get('patched_content', ''),
                self._store.state.settings
            )
            QTimer.singleShot(0, lambda: callback(verdict))
        AsyncRuntime.run_coroutine(task())

    def apply_prepared_patches(self, prepared_patches: list):
        self._patch_uc.apply_prepared(prepared_patches)

    def parse_error_log(self, text: str):
        state = self._store.state
        if not state.scanned_files_paths:
            self._dispatcher.dispatch(UI_ADD_LOG, "⚠️ Сначала отсканируйте файлы!")
            return
        matched = []
        for path in state.scanned_files_paths:
            basename = os.path.basename(path)
            if basename in text:
                matched.append(path)
        if not matched:
            self._dispatcher.dispatch(UI_ADD_LOG, "⚠️ В логе не найдены файлы проекта.")
            return
        self._dispatcher.dispatch(EXCLUSION_CLEAR, None)
        all_paths = set(state.scanned_files_paths)
        for p in (all_paths - set(matched)):
            self._dispatcher.dispatch(EXCLUSION_ADD, p)
        self._dispatcher.dispatch(UI_ADD_LOG, f"🎯 Найдено {len(matched)} файлов из лога ошибки!")

    def show_tour(self):
        steps = self._tour_service.get_tour_steps()
        self._dispatcher.dispatch(UI_SHOW_TOUR, steps)

    def close_tour(self):
        self._dispatcher.dispatch(UI_CLOSE_TOUR, None)

    def copy_to_clipboard(self, text: str):
        self._output_service.copy_to_clipboard(text)
        self._dispatcher.dispatch(UI_ADD_LOG, "📋 Текст скопирован в буфер обмена")

    def get_search_markers_for_preview(self, filepath: str) -> List[str]:
        return self._format_service.get_search_markers(filepath)

    def generate_html_diff(self, source_text: str, target_text: str, colors: dict, fonts: dict) -> str:
        return self._format_service.generate_html_diff(source_text, target_text, colors, fonts)

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path: return ""
        clean = path
        while True:
            prev = clean
            clean = clean.strip().strip('"\'')
            if clean == prev:
                break
        try:
            return os.path.normpath(clean)
        except (TypeError, ValueError, OSError):
            return clean