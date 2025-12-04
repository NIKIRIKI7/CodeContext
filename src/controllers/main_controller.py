import threading
import os
from typing import Tuple

from ..store.store import Store
from ..store.state import ProcessedFile
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import *
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT

# Импорт сервисов
from ..services.file_service import FileService
from ..services.processing_service import ProcessingService
from ..services.cleaner_service import CleanerService
from ..services.token_service import TokenService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..services.integration_service import IntegrationService

# Data Layer
from ..data.file_system_repository import FileSystemRepository
from ..data.settings_repository import SettingsRepository


class MainController:
    """
    Контроллер для GUI режима.
    Управляет бизнес-логикой, запускает потоки и взаимодействует с сервисами.
    """

    def __init__(self, store: Store, dispatcher: Dispatcher):
        self.store = store
        self.dispatcher = dispatcher

        # --- Dependency Injection ---
        fs_repo = FileSystemRepository()
        self.settings_repo = SettingsRepository()

        self.file_service = FileService(fs_repo)
        self.process_service = ProcessingService(fs_repo)
        self.cleaner_service = CleanerService()
        self.token_service = TokenService()
        self.format_service = FormattingService()
        self.output_service = OutputService()
        self.integration_service = IntegrationService()

    def load_initial_settings(self):
        """Загрузка настроек при старте"""
        data = self.settings_repo.load()
        if not data:
            data = {
                'extensions': PRESETS['Default']['ext'],
                'ignored_paths': PRESETS['Default']['ign'],
                'system_prompt': DEFAULT_SYSTEM_PROMPT
            }
        self.dispatcher.dispatch(SETTINGS_LOADED, data)

    def update_settings(self, settings_data: dict):
        """Обновление настроек в стейте"""
        self.dispatcher.dispatch(SETTINGS_UPDATE, settings_data)

    def save_settings(self):
        """Сохранение текущих настроек в файл"""
        self.settings_repo.save(self.store.state.settings.__dict__)

    def reset_settings(self):
        """Сброс настроек к дефолтным"""
        default_data = {
            'extensions': PRESETS['Default']['ext'],
            'ignored_paths': PRESETS['Default']['ign'],
            'system_prompt': DEFAULT_SYSTEM_PROMPT,
            'minify': True,
            'remove_comments': True,
            'remove_secrets': True,
            'include_tree': True,
            'cli_minify': True,
            'cli_remove_comments': True,
            'cli_remove_secrets': True,
            'cli_include_tree': True,
            'cli_format': "plain"
        }
        self.dispatcher.dispatch(SETTINGS_UPDATE, default_data)
        self.settings_repo.save(default_data)
        self.dispatcher.dispatch(UI_ADD_LOG, "Настройки сброшены")

    def apply_preset(self, preset_name: str):
        """Применение пресета расширений"""
        preset = PRESETS.get(preset_name)
        if preset:
            self.dispatcher.dispatch(SETTINGS_UPDATE, {
                'extensions': preset['ext'],
                'ignored_paths': preset['ign']
            })

    def add_folder(self, path: str):
        self.dispatcher.dispatch(FOLDER_ADD, path)

    def clear_folders(self):
        self.dispatcher.dispatch(FOLDER_CLEAR)

    def install_context_menu(self) -> Tuple[bool, str]:
        return self.integration_service.install_context_menu()

    def remove_context_menu(self) -> Tuple[bool, str]:
        return self.integration_service.remove_context_menu()

    def start_processing(self, target_type: str, save_path: str = None):
        """Запуск воркера обработки в отдельном потоке"""
        if not self.store.state.selected_folders:
            return False, "Выберите папки для сканирования"

        threading.Thread(
            target=self._worker,
            args=(target_type, save_path),
            daemon=True
        ).start()
        return True, ""

    def _worker(self, target: str, save_path: str = None):
        """
        Основной пайплайн обработки данных.
        Выполняется в фоновом потоке.
        """
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование...", 'progress': 0.1})
        self.dispatcher.dispatch(UI_ADD_LOG, "Начало работы...")

        state = self.store.state

        try:
            # 1. SCAN
            files_paths = self.file_service.scan_folders(
                state.selected_folders,
                state.settings.extensions,
                state.settings.ignored_paths,
                state.settings.use_git
            )

            if not files_paths:
                self.dispatcher.dispatch(SCAN_FAILURE, "Файлы не найдены")
                self.dispatcher.dispatch(UI_SET_LOADING, False)
                return

            self.dispatcher.dispatch(SCAN_SUCCESS, files_paths)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Чтение файлов...", 'progress': 0.2})

            # 2. READ
            raw_files = self.process_service.read_files(files_paths)

            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Обработка...", 'progress': 0.4})

            # 3. PROCESS LOOP
            processed_results = []

            for i, raw_file in enumerate(raw_files):
                if i % 10 == 0:
                    prog = 0.4 + (0.4 * (i / len(raw_files)))
                    self.dispatcher.dispatch(UI_UPDATE_STATUS,
                                             {'message': f"Обработка {i}/{len(raw_files)}...", 'progress': prog})

                content = raw_file['content']
                ext = raw_file['ext']

                # Используем настройки из Store
                cleaned_content = self.cleaner_service.clean(content, ext, state.settings)
                tokens = self.token_service.count_tokens(cleaned_content)

                processed_results.append(ProcessedFile(
                    path=raw_file['path'],
                    content=cleaned_content,
                    tokens=tokens
                ))

            self.dispatcher.dispatch(PROCESSING_SUCCESS, processed_results)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Форматирование...", 'progress': 0.9})

            # 4. FORMAT
            text_result = self.format_service.format_output(
                processed_results,
                state.settings.output_format,
                state.settings.include_tree,
                state.settings.system_prompt
            )

            total_tokens = sum(f.tokens for f in processed_results)
            self.dispatcher.dispatch(FORMATTING_SUCCESS, {'text': text_result, 'tokens': total_tokens})

            # 5. OUTPUT
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

            # Сохраняем настройки после успешного прогона
            self.save_settings()

        except Exception as e:
            self.dispatcher.dispatch(UI_ADD_LOG, f"CRITICAL ERROR: {e}")
            import traceback
            traceback.print_exc()
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Готово", 'progress': 1.0})