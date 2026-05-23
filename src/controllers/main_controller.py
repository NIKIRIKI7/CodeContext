import os
import asyncio
import traceback
from typing import Tuple, List
from ..store.store import Store
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import *
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT
from ..utils.async_runtime import AsyncRuntime
from ..services.file_service import FileService
from ..services.processing_service import ProcessingService
from ..services.cleaner_service import CleanerService
from ..services.token_service import TokenService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..services.integration_service import IntegrationService
from ..services.skeleton_service import SkeletonService
from ..services.github_service import GitHubService
from ..services.dependency_service import DependencyService
from ..data.file_system_repository import FileSystemRepository
from ..data.settings_repository import SettingsRepository
from ..utils.pipeline_utils import PipelineUtils
from ..store.state import ProcessedFile


class MainController:
    def __init__(self, store: Store, dispatcher: Dispatcher):
        self.store = store
        self.dispatcher = dispatcher
        self.fs_repo = FileSystemRepository()
        self.settings_repo = SettingsRepository()
        self.file_service = FileService(self.fs_repo)
        self.process_service = ProcessingService(self.fs_repo)
        self.cleaner_service = CleanerService()
        self.skeleton_service = SkeletonService()
        self.token_service = TokenService()
        self.format_service = FormattingService()
        self.output_service = OutputService()
        self.integration_service = IntegrationService()
        self.github_service = GitHubService()
        self.dependency_service = DependencyService()

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path:
            return ""
        clean_path = path
        while True:
            prev = clean_path
            clean_path = clean_path.strip().strip('"\'')
            if clean_path == prev:
                break
        try:
            clean_path = os.path.normpath(clean_path)
        except (TypeError, ValueError, OSError):
            pass
        return clean_path

    def load_initial_settings(self):
        data = self.settings_repo.load()
        if not data:
            data = {
                'extensions': PRESETS['Default']['ext'],
                'ignored_paths': PRESETS['Default']['ign'],
                'system_prompt': DEFAULT_SYSTEM_PROMPT
            }
        self.dispatcher.dispatch(SETTINGS_LOADED, data)

    def update_settings(self, settings_data: dict):
        self.dispatcher.dispatch(SETTINGS_UPDATE, settings_data)

    def save_settings(self):
        self.settings_repo.save(self.store.state.settings.__dict__)

    def reset_settings(self):
        default_data = {
            'extensions': PRESETS['Default']['ext'],
            'ignored_paths': PRESETS['Default']['ign'],
            'system_prompt': DEFAULT_SYSTEM_PROMPT,
            'minify': True, 'remove_comments': True, 'remove_secrets': True,
            'include_tree': True, 'include_dependencies': False,
            'skeleton_mode': False, 'use_git': False, 'use_gitignore': True,
            'cli_minify': True, 'cli_remove_comments': True, 'cli_remove_secrets': True,
            'cli_include_tree': True, 'cli_skeleton_mode': False, 'cli_use_gitignore': True, 'cli_format': "plain",
            'python_interpreter': ""
        }
        self.dispatcher.dispatch(SETTINGS_UPDATE, default_data)
        self.settings_repo.save(default_data)
        self.dispatcher.dispatch(UI_ADD_LOG, "Настройки сброшены")

    def apply_preset(self, preset_name: str):
        preset = PRESETS.get(preset_name)
        if preset:
            self.dispatcher.dispatch(SETTINGS_UPDATE, {'extensions': preset['ext'], 'ignored_paths': preset['ign']})

    def add_folder(self, path: str):
        clean_path = self._normalize_path(path)
        if clean_path and os.path.exists(clean_path):
            self.dispatcher.dispatch(FOLDER_ADD, clean_path)

    def remove_folder(self, path: str):
        self.dispatcher.dispatch(FOLDER_REMOVE, path)

    def edit_folder(self, old_path: str, new_path: str):
        clean_new = self._normalize_path(new_path)
        if old_path != clean_new and clean_new:
            self.dispatcher.dispatch(FOLDER_UPDATE, {'old': old_path, 'new': clean_new})

    def clear_folders(self):
        temp_folders = self.store.state.temp_folders
        if temp_folders:
            self.dispatcher.dispatch(UI_ADD_LOG, "Очистка временных файлов...")
            AsyncRuntime.run_coroutine(self._clear_folders_async(temp_folders))
        else:
            self.dispatcher.dispatch(FOLDER_CLEAR)

    async def _clear_folders_async(self, folders):
        for folder in folders:
            await self.fs_repo.delete_directory_async(folder)
        self.dispatcher.dispatch(FOLDER_CLEAR)

    def install_context_menu(self) -> Tuple[bool, str]:
        return self.integration_service.install_context_menu()

    def remove_context_menu(self) -> Tuple[bool, str]:
        return self.integration_service.remove_context_menu()

    def add_github_repo(self, url: str):
        if not url: return
        AsyncRuntime.run_coroutine(self._github_worker_async(url))

    async def _github_worker_async(self, url: str):
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Клонирование...", 'progress': 0.0})
        self.dispatcher.dispatch(UI_ADD_LOG, f"GitHub Cloning: {url}")
        try:
            temp_path = await self.github_service.clone_repo_async(url)
            self.dispatcher.dispatch(GITHUB_CLONE_SUCCESS, temp_path)
            self.dispatcher.dispatch(UI_ADD_LOG, "Репозиторий загружен")
        except Exception as e:
            self.dispatcher.dispatch(GITHUB_CLONE_FAILURE, str(e))
            self.dispatcher.dispatch(UI_ADD_LOG, f"GitHub Error: {e}")
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Готово", 'progress': 0.0})

    def toggle_file_exclusion(self, path: str, is_included: bool):
        """Коллбек от дерева файлов: пользователь поменял галочку"""
        if is_included:
            # Если включили (поставили галочку), убираем из списка исключений
            self.dispatcher.dispatch(EXCLUSION_REMOVE, path)
        else:
            # Если выключили (сняли галочку), добавляем в исключения
            self.dispatcher.dispatch(EXCLUSION_ADD, path)

    def scan_only(self):
        """Только сканирование (для предпросмотра)"""
        if not self.store.state.selected_folders:
            self.dispatcher.dispatch(UI_ADD_LOG, "⚠️ Выберите папки для сканирования")
            return

        AsyncRuntime.run_coroutine(self._scan_worker_async())

    async def _scan_worker_async(self):
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование...", 'progress': 0.0})
        self.dispatcher.dispatch(UI_ADD_LOG, "Начало сканирования...")

        state = self.store.state
        try:
            files_paths = await self.file_service.scan_folders_async(
                state.selected_folders,
                state.settings.extensions,
                state.settings.ignored_paths,
                state.settings.use_git,
                state.settings.use_gitignore
            )

            if not files_paths:
                self.dispatcher.dispatch(SCAN_FAILURE, "Файлы не найдены")
                self.dispatcher.dispatch(UI_ADD_LOG, "⚠️ Файлы не найдены")
            else:
                self.dispatcher.dispatch(SCAN_SUCCESS, files_paths)
                self.dispatcher.dispatch(UI_ADD_LOG, f"Найдено файлов: {len(files_paths)}")

        except Exception as e:
            self.dispatcher.dispatch(SCAN_FAILURE, str(e))
            self.dispatcher.dispatch(UI_ADD_LOG, f"Error: {e}")
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование завершено", 'progress': 0.0})

    def start_processing(self, target_type: str, save_path: str = None):
        if not self.store.state.selected_folders:
            return False, "Выберите папки или URL для сканирования"

        # Если файлы еще не сканированы (пустой список), сканируем сначала
        if not self.store.state.scanned_files_paths:
            AsyncRuntime.run_coroutine(self._scan_and_process_worker_async(target_type, save_path))
        else:
            # Если уже сканированы (и, возможно, отфильтрованы в дереве), сразу обрабатываем
            AsyncRuntime.run_coroutine(self._process_worker_async(target_type, save_path))

        return True, ""

    async def _scan_and_process_worker_async(self, target: str, save_path: str = None):
        """Полный цикл: Скан -> Фильтр -> Процессинг"""
        await self._scan_worker_async()
        # Если скан успешен, продолжаем
        if self.store.state.scanned_files_paths:
            await self._process_worker_async(target, save_path)

    async def _process_worker_async(self, target: str, save_path: str = None):
        """Обработка уже найденных файлов с учетом ручных исключений"""
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        state = self.store.state

        # Фильтрация на основе manual_exclusions
        all_files = state.scanned_files_paths
        exclusions = state.manual_exclusions

        # Оставляем только те, которых НЕТ в исключениях
        files_to_process = [p for p in all_files if p not in exclusions]

        if not files_to_process:
            self.dispatcher.dispatch(UI_ADD_LOG, "⚠️ Нет файлов для обработки (все исключены?)")
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            return

        self.dispatcher.dispatch(UI_UPDATE_STATUS,
                                 {'message': f"Чтение {len(files_to_process)} файлов...", 'progress': 0.2})
        self.dispatcher.dispatch(UI_ADD_LOG, f"Обработка {len(files_to_process)} файлов...")

        try:
            raw_files = await self.process_service.read_files_async(files_to_process)

            dependency_map = None
            if state.settings.include_dependencies:
                self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Анализ зависимостей...", 'progress': 0.3})
                dependency_map = await self.dependency_service.resolve_dependencies(raw_files)

            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Обработка и минификация...", 'progress': 0.5})
            processed_results = await asyncio.to_thread(self._cpu_heavy_processing, raw_files, state.settings)

            self.dispatcher.dispatch(PROCESSING_SUCCESS, processed_results)

            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Форматирование...", 'progress': 0.9})
            text_result = await asyncio.to_thread(
                self._format_output,
                processed_results,
                state.settings,
                dependency_map
            )

            total_tokens = self._count_total_tokens(processed_results)
            self.dispatcher.dispatch(FORMATTING_SUCCESS, {'text': text_result, 'tokens': total_tokens})

            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сохранение...", 'progress': 0.95})

            if target == 'clipboard':
                self.output_service.copy_to_clipboard(text_result)
                self.dispatcher.dispatch(UI_ADD_LOG, "Скопировано в буфер обмена")
            elif target == 'file' and save_path:
                self.output_service.save_to_file(text_result, save_path)
                self.dispatcher.dispatch(UI_ADD_LOG, f"Сохранено в {save_path}")
            elif target == 'pdf' and save_path:
                await asyncio.to_thread(self.output_service.save_to_pdf, text_result, save_path)
                self.dispatcher.dispatch(UI_ADD_LOG, f"PDF создан: {save_path}")

            self.save_settings()

        except Exception as e:
            self.dispatcher.dispatch(UI_ADD_LOG, f"CRITICAL ERROR: {e}")
            traceback.print_exc()
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Готово", 'progress': 1.0})

    def _cpu_heavy_processing(self, raw_files, settings):
        """Выполняется в отдельном потоке"""
        import copy
        proc_settings = copy.copy(settings)
        if proc_settings.skeleton_mode:
            proc_settings.minify = False
        return PipelineUtils.process_files_batch(
            raw_files=raw_files,
            options=proc_settings,
            cleaner_service=self.cleaner_service,
            skeleton_service=self.skeleton_service,
            token_service=self.token_service
        )

    def _format_output(self, processed_results, settings, dep_map) -> str:
        """Только форматирование текста"""
        return self.format_service.format_output(
            files=processed_results,
            fmt=settings.output_format,
            include_tree=settings.include_tree,
            system_prompt=settings.system_prompt,
            dependency_map=dep_map,
            template_path=settings.template_path
        )

    @staticmethod
    def _count_total_tokens(processed_results: List[ProcessedFile]) -> int:
        """Только подсчет токенов"""
        return sum(f.tokens for f in processed_results)