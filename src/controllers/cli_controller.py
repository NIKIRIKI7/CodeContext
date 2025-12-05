import time
import os
import sys
from types import SimpleNamespace
from typing import Dict, List
from ..data.settings_repository import SettingsRepository
from ..data.file_system_repository import FileSystemRepository
from ..services.file_service import FileService
from ..services.processing_service import ProcessingService
from ..services.cleaner_service import CleanerService
from ..services.token_service import TokenService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..store.state import ProcessedFile
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT


class CliController:
    """
    Контроллер для обработки запросов в режиме командной строки (Headless).
    Управляет потоком данных: Settings -> Services -> Output.
    """

    def __init__(self):
        self.settings_repo = SettingsRepository()
        self.fs_repo = FileSystemRepository()
        self.file_service = FileService(self.fs_repo)
        self.process_service = ProcessingService(self.fs_repo)
        self.cleaner_service = CleanerService()
        self.token_service = TokenService()
        self.format_service = FormattingService()
        self.output_service = OutputService()

    def run(self, target_path: str):
        """Основной метод запуска CLI пайплайна"""
        # 1. Нормализация пути (удаляем кавычки, делаем абсолютным)
        target_path = os.path.abspath(target_path.strip('"\''))

        print(f"\n🚀 CodeContext AI: Запуск...")
        print(f"📂 Цель: {target_path}")

        if not os.path.exists(target_path):
            print(f"❌ Ошибка: Путь не существует.")
            self._keep_window_open()
            return

        # 2. Загрузка конфигурации с фоллбэками
        config = self._load_config()

        # Защита от пустых расширений в конфиге
        extensions = config.get('extensions', '')
        if not extensions or not extensions.strip():
            extensions = PRESETS['Default']['ext']

        options = SimpleNamespace(
            minify=config.get('cli_minify', True),
            remove_comments=config.get('cli_remove_comments', True),
            remove_secrets=config.get('cli_remove_secrets', True),
            extensions=extensions,
            ignored_paths=config.get('ignored_paths', PRESETS['Default']['ign']),
            use_git=config.get('use_git', False)  # Исправлено: читаем из конфига
        )

        output_format = config.get('cli_format', 'plain')
        include_tree = config.get('cli_include_tree', True)
        system_prompt = config.get('system_prompt', DEFAULT_SYSTEM_PROMPT)

        try:
            # 3. Поиск файлов
            print(f"🔍 Сканирование ({'Git' if options.use_git else 'FS'})...", end=" ")

            # Логика определения: файл или папка обрабатывается внутри service или здесь
            # Передаем список, так как service принимает список
            file_paths = self.file_service.scan_folders(
                [target_path],
                options.extensions,
                options.ignored_paths,
                options.use_git
            )

            if not file_paths:
                print("\n⚠️  Файлы не найдены.")
                print(f"   Проверьте расширения: {options.extensions}")
                if options.use_git:
                    print("   (Включен режим Git: убедитесь, что файлы отслеживаются или изменены)")
                return

            print(f"✅ Найдено: {len(file_paths)}")

            # 4. Чтение
            print("📖 Чтение файлов...", end=" ")
            raw_files = self.process_service.read_files(file_paths)
            print(f"✅ Успешно: {len(raw_files)}")

            # 5. Обработка
            print("⚙️  Обработка...", end=" ")
            processed_files = []
            for raw in raw_files:
                cleaned = self.cleaner_service.clean(raw['content'], raw['ext'], options)
                tokens = self.token_service.count_tokens(cleaned)
                processed_files.append(ProcessedFile(
                    path=raw['path'],
                    content=cleaned,
                    tokens=tokens
                ))
            print("✅ Готово")

            # 6. Форматирование
            final_text = self.format_service.format_output(
                processed_files,
                output_format,
                include_tree,
                system_prompt
            )

            total_tokens = sum(f.tokens for f in processed_files)
            print(f"📊 Всего токенов: ~{total_tokens}")

            # 7. Вывод
            self.output_service.copy_to_clipboard(final_text)
            print(f"📋 Результат ({output_format}) скопирован в буфер обмена!")

        except Exception as e:
            print(f"\n🔥 Критическая ошибка: {e}")
            import traceback
            traceback.print_exc()
        finally:
            self._keep_window_open()

    def _load_config(self) -> Dict:
        """Загружает настройки или возвращает дефолтные"""
        cfg = self.settings_repo.load()
        if not cfg:
            return {}
        return cfg

    def _keep_window_open(self):
        """Задержка перед закрытием консоли"""
        print("\n(Окно закроется через 3 секунды...)")
        time.sleep(3)