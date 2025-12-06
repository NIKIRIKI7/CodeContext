import time
import os
import asyncio
import traceback
from types import SimpleNamespace
from typing import Dict, List, Tuple, Set, Optional

from ..data.settings_repository import SettingsRepository
from ..data.file_system_repository import FileSystemRepository
from ..services.file_service import FileService
from ..services.processing_service import ProcessingService
from ..services.cleaner_service import CleanerService
from ..services.skeleton_service import SkeletonService
from ..services.token_service import TokenService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..services.dependency_service import DependencyService  # <--- Добавлен импорт
from ..store.state import ProcessedFile
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT
from ..utils.pipeline_utils import PipelineUtils


class CliController:
    """
    Контроллер для обработки запросов в режиме командной строки (Headless).
    Управляет потоком данных: Settings -> Services -> Output.
    """

    def __init__(self):
        # Инициализация репозиториев
        self.settings_repo = SettingsRepository()
        self.fs_repo = FileSystemRepository()

        # Инициализация сервисов
        self.file_service = FileService(self.fs_repo)
        self.process_service = ProcessingService(self.fs_repo)
        self.cleaner_service = CleanerService()
        self.skeleton_service = SkeletonService()
        self.token_service = TokenService()
        self.format_service = FormattingService()
        self.output_service = OutputService()
        self.dependency_service = DependencyService()  # <--- Инициализация сервиса зависимостей

    def run(self, target_path: str):
        """Основной метод запуска CLI пайплайна"""
        target_path = self._normalize_path(target_path)

        print(f"\n🚀 CodeContext AI: Запуск...")
        print(f"📂 Цель: {target_path}")

        if not self._validate_target(target_path):
            return

        # 1. Загрузка конфигурации
        config = self._load_config()
        options, system_prompt, output_format = self._prepare_options(config)

        try:
            # 2. Сканирование файлов
            file_paths = self._step_scan_files(target_path, options)
            if not file_paths:
                self._exit_with_message("⚠️ Файлы не найдены.")
                return

            # 3. Чтение файлов
            raw_files = self._step_read_files(file_paths)

            # 4. Анализ зависимостей (НОВЫЙ ШАГ)
            dependency_map = self._step_resolve_dependencies(raw_files, options)

            # 5. Обработка (Очистка -> Скелет -> Токены)
            processed_files = self._step_process_files(raw_files, options)

            # 6. Форматирование и вывод (Передаем dependency_map)
            self._step_output(processed_files, output_format, options.include_tree, system_prompt, dependency_map)

        except Exception as e:
            self._handle_error(e)
        finally:
            self._keep_window_open()

    # --- Вспомогательные методы логики (Steps) ---

    def _step_scan_files(self, path: str, options: SimpleNamespace) -> List[str]:
        """Шаг сканирования файловой системы"""
        print(f"🔍 Сканирование ({'Git' if options.use_git else 'FS'})...", end=" ")

        file_paths = asyncio.run(self.file_service.scan_folders_async(
            [path],
            options.extensions,
            options.ignored_paths,
            options.use_git,
            options.use_gitignore
        ))

        print(f"✅ Найдено: {len(file_paths)}")
        return file_paths

    def _step_read_files(self, file_paths: List[str]) -> List[Dict]:
        """Шаг чтения содержимого файлов"""
        print("📖 Чтение файлов...", end=" ")

        raw_files = asyncio.run(self.process_service.read_files_async(file_paths))

        print(f"✅ Успешно: {len(raw_files)}")
        return raw_files

    def _step_resolve_dependencies(self, raw_files: List[Dict], options: SimpleNamespace) -> Optional[Dict[str, Set[str]]]:
        """Шаг построения графа зависимостей"""
        if not options.include_dependencies:
            return None

        print("🕸  Анализ зависимостей...", end=" ")
        try:
            # Запускаем асинхронный метод в синхронном контексте
            dependency_map = asyncio.run(self.dependency_service.resolve_dependencies(raw_files))
            print(f"✅ (Файлов со связями: {len(dependency_map)})")
            return dependency_map
        except Exception as e:
            print(f"❌ Ошибка: {e}")
            return None

    def _step_process_files(self, raw_files: List[Dict], options: SimpleNamespace) -> List[ProcessedFile]:
        """Шаг обработки контента (использует PipelineUtils)"""
        print("⚙️  Обработка...", end=" ")

        processed_files = PipelineUtils.process_files_batch(
            raw_files=raw_files,
            options=options,
            cleaner_service=self.cleaner_service,
            skeleton_service=self.skeleton_service,
            token_service=self.token_service
        )

        print("✅ Готово")
        return processed_files

    def _step_output(self,
                     processed_files: List[ProcessedFile],
                     fmt: str,
                     include_tree: bool,
                     prompt: str,
                     dependency_map: Optional[Dict[str, Set[str]]] = None):
        """Шаг форматирования и сохранения"""
        final_text = self.format_service.format_output(
            files=processed_files,
            fmt=fmt,
            include_tree=include_tree,
            system_prompt=prompt,
            dependency_map=dependency_map  # <--- Передаем карту зависимостей
        )

        total_tokens = sum(f.tokens for f in processed_files)
        print(f"📊 Всего токенов: ~{total_tokens}")

        self.output_service.copy_to_clipboard(final_text)
        print(f"📋 Результат ({fmt}) скопирован в буфер обмена!")

    # --- Вспомогательные методы конфигурации ---

    def _normalize_path(self, path: str) -> str:
        if not path: return ""
        return os.path.abspath(path.strip('"\''))

    def _validate_target(self, path: str) -> bool:
        if not os.path.exists(path):
            print(f"❌ Ошибка: Путь не существует.")
            self._keep_window_open()
            return False
        return True

    def _load_config(self) -> Dict:
        cfg = self.settings_repo.load()
        return cfg if cfg else {}

    def _prepare_options(self, config: Dict) -> Tuple[SimpleNamespace, str, str]:
        """Подготовка объекта опций из сырого конфига"""
        # Расширения
        extensions = config.get('extensions', '')
        if not extensions or not extensions.strip():
            extensions = PRESETS['Default']['ext']

        # Режимы
        skeleton_mode = config.get('cli_skeleton_mode', False)
        minify = config.get('cli_minify', True)
        if skeleton_mode:
            minify = False

        # Объект опций для передачи в сервисы
        options = SimpleNamespace(
            minify=minify,
            remove_comments=config.get('cli_remove_comments', True),
            remove_secrets=config.get('cli_remove_secrets', True),
            skeleton_mode=skeleton_mode,
            extensions=extensions,
            ignored_paths=config.get('ignored_paths', PRESETS['Default']['ign']),
            use_git=config.get('use_git', False),
            use_gitignore=config.get('cli_use_gitignore', True),
            include_tree=config.get('cli_include_tree', True),
            # Важно: берем настройку зависимостей из конфига (по умолчанию False, если не задано)
            include_dependencies=config.get('include_dependencies', False)
        )

        system_prompt = config.get('system_prompt', DEFAULT_SYSTEM_PROMPT)
        output_format = config.get('cli_format', 'plain')

        return options, system_prompt, output_format

    # --- Управление окном и ошибками ---

    def _handle_error(self, e: Exception):
        print(f"\n🔥 Критическая ошибка: {e}")
        traceback.print_exc()

    def _exit_with_message(self, msg: str):
        print(f"\n{msg}")
        self._keep_window_open()

    def _keep_window_open(self):
        print("\n(Окно закроется через 3 секунды...)")
        time.sleep(3)