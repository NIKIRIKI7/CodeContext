import threading
import copy
import os
from typing import Tuple
from ..store.store import Store
from ..store.state import ProcessedFile
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import *
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT
from ..services.file_service import FileService
from ..services.processing_service import ProcessingService
from ..services.cleaner_service import CleanerService
from ..services.token_service import TokenService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..services.integration_service import IntegrationService
from ..services.skeleton_service import SkeletonService
from ..services.github_service import GitHubService
from ..data.file_system_repository import FileSystemRepository
from ..data.settings_repository import SettingsRepository


class MainController:
    """
    Контроллер для GUI режима.
    """

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

    def _normalize_path(self, path: str) -> str:
        """
        Мощная нормализация пути:
        1. Удаляет пробелы по краям.
        2. Удаляет кавычки.
        3. Удаляет возможные пробелы, оставшиеся после удаления кавычек.
        4. Приводит слеши к системному стандарту.
        """
        if not path:
            return ""

        # Шаг 1: Удаляем пробелы и кавычки в цикле, пока строка меняется
        # Это защищает от ситуаций вида ' "Path" ' или " 'Path' "
        clean_path = path
        while True:
            prev = clean_path
            clean_path = clean_path.strip()
            clean_path = clean_path.strip('"\'')
            if clean_path == prev:
                break

        # Шаг 2: Нормализуем путь (меняет / на \ в Windows)
        # Также убирает лишние слеши в конце
        try:
            clean_path = os.path.normpath(clean_path)
        except:
            pass  # Если путь совсем битый, оставляем как есть

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
            'minify': True,
            'remove_comments': True,
            'remove_secrets': True,
            'include_tree': True,
            'skeleton_mode': False,
            'use_gitignore': True,
            'cli_minify': True,
            'cli_remove_comments': True,
            'cli_remove_secrets': True,
            'cli_include_tree': True,
            'cli_skeleton_mode': False,
            'cli_use_gitignore': True,
            'cli_format': "plain"
        }
        self.dispatcher.dispatch(SETTINGS_UPDATE, default_data)
        self.settings_repo.save(default_data)
        self.dispatcher.dispatch(UI_ADD_LOG, "Настройки сброшены")

    def apply_preset(self, preset_name: str):
        preset = PRESETS.get(preset_name)
        if preset:
            self.dispatcher.dispatch(SETTINGS_UPDATE, {
                'extensions': preset['ext'],
                'ignored_paths': preset['ign']
            })

    def add_folder(self, path: str):
        """Добавление папки с нормализацией пути"""
        clean_path = self._normalize_path(path)
        if clean_path and os.path.exists(clean_path):
            self.dispatcher.dispatch(FOLDER_ADD, clean_path)
        elif clean_path:
            # Если путь нормализовался, но не существует - логируем или игнорируем
            print(f"Warning: Path does not exist: {clean_path}")

    def remove_folder(self, path: str):
        self.dispatcher.dispatch(FOLDER_REMOVE, path)

    def edit_folder(self, old_path: str, new_path: str):
        """Редактирование папки с нормализацией"""
        clean_new = self._normalize_path(new_path)
        if old_path != clean_new and clean_new:
            self.dispatcher.dispatch(FOLDER_UPDATE, {'old': old_path, 'new': clean_new})

    def add_github_repo(self, url: str):
        if not url: return
        threading.Thread(target=self._github_worker, args=(url,), daemon=True).start()

    def _github_worker(self, url: str):
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Клонирование репозитория...", 'progress': 0.0})
        self.dispatcher.dispatch(UI_ADD_LOG, f"GitHub Cloning: {url}")

        try:
            temp_path = self.github_service.clone_repo(url)
            self.dispatcher.dispatch(GITHUB_CLONE_SUCCESS, temp_path)
            self.dispatcher.dispatch(UI_ADD_LOG, "Репозиторий успешно загружен")
        except Exception as e:
            self.dispatcher.dispatch(GITHUB_CLONE_FAILURE, str(e))
            self.dispatcher.dispatch(UI_ADD_LOG, f"GitHub Error: {e}")
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Готово", 'progress': 0.0})

    def clear_folders(self):
        temp_folders = self.store.state.temp_folders
        if temp_folders:
            self.dispatcher.dispatch(UI_ADD_LOG, "Очистка временных файлов...")
            for folder in temp_folders:
                self.fs_repo.delete_directory(folder)

        self.dispatcher.dispatch(FOLDER_CLEAR)

    def install_context_menu(self) -> Tuple[bool, str]:
        return self.integration_service.install_context_menu()

    def remove_context_menu(self) -> Tuple[bool, str]:
        return self.integration_service.remove_context_menu()

    def start_processing(self, target_type: str, save_path: str = None):
        if not self.store.state.selected_folders:
            return False, "Выберите папки или URL для сканирования"

        threading.Thread(
            target=self._worker,
            args=(target_type, save_path),
            daemon=True
        ).start()
        return True, ""

    def _worker(self, target: str, save_path: str = None):
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование...", 'progress': 0.1})
        self.dispatcher.dispatch(UI_ADD_LOG, "Начало работы...")

        state = self.store.state

        try:
            files_paths = self.file_service.scan_folders(
                state.selected_folders,
                state.settings.extensions,
                state.settings.ignored_paths,
                state.settings.use_git,
                state.settings.use_gitignore
            )

            if not files_paths:
                self.dispatcher.dispatch(SCAN_FAILURE, "Файлы не найдены")
                self.dispatcher.dispatch(UI_SET_LOADING, False)
                return

            self.dispatcher.dispatch(SCAN_SUCCESS, files_paths)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Чтение файлов...", 'progress': 0.2})

            raw_files = self.process_service.read_files(files_paths)

            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Обработка...", 'progress': 0.4})

            processed_results = []

            processing_settings = copy.copy(state.settings)
            if state.settings.skeleton_mode:
                processing_settings.minify = False

            for i, raw_file in enumerate(raw_files):
                if i % 10 == 0:
                    prog = 0.4 + (0.4 * (i / len(raw_files)))
                    self.dispatcher.dispatch(UI_UPDATE_STATUS,
                                             {'message': f"Обработка {i}/{len(raw_files)}...", 'progress': prog})

                content = raw_file['content']
                ext = raw_file['ext']

                cleaned_content = self.cleaner_service.clean(content, ext, processing_settings)

                if state.settings.skeleton_mode:
                    cleaned_content = self.skeleton_service.make_skeleton(cleaned_content, ext)

                tokens = self.token_service.count_tokens(cleaned_content)

                processed_results.append(ProcessedFile(
                    path=raw_file['path'],
                    content=cleaned_content,
                    tokens=tokens
                ))

            self.dispatcher.dispatch(PROCESSING_SUCCESS, processed_results)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Форматирование...", 'progress': 0.9})

            text_result = self.format_service.format_output(
                processed_results,
                state.settings.output_format,
                state.settings.include_tree,
                state.settings.system_prompt
            )

            total_tokens = sum(f.tokens for f in processed_results)
            self.dispatcher.dispatch(FORMATTING_SUCCESS, {'text': text_result, 'tokens': total_tokens})
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сохранение...", 'progress': 0.95})

            if target == 'clipboard':
                self.output_service.copy_to_clipboard(text_result)
                self.dispatcher.dispatch(UI_ADD_LOG, "Скопировано в буфер обмена")
            elif target == 'file' and save_path:
                self.output_service.save_to_file(text_result, save_path)
                self.dispatcher.dispatch(UI_ADD_LOG, f"Сохранено в {save_path}")
            elif target == 'pdf' and save_path:
                self.output_service.save_to_pdf(text_result, save_path)
                self.dispatcher.dispatch(UI_ADD_LOG, f"PDF создан: {save_path}")

            self.save_settings()

        except Exception as e:
            self.dispatcher.dispatch(UI_ADD_LOG, f"CRITICAL ERROR: {e}")
            import traceback
            traceback.print_exc()
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Готово", 'progress': 1.0})