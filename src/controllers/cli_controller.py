import os
import asyncio
import time
import traceback
from ..store.store import Store
from ..actions.dispatcher import Dispatcher
from ..data.settings_repository import SettingsRepository
from ..use_cases.scan_use_case import ScanWorkspaceUseCase
from ..use_cases.process_use_case import ProcessWorkspaceUseCase
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT
from ..utils.logger import app_logger


class CliController:
    """CLI-точка входа. Инициализирует Store и вызывает Use Cases."""

    def __init__(
            self,
            store: Store,
            dispatcher: Dispatcher,
            settings_repo: SettingsRepository,
            scan_use_case: ScanWorkspaceUseCase,
            process_use_case: ProcessWorkspaceUseCase,
    ):
        self._store = store
        self._dispatcher = dispatcher
        self._settings_repo = settings_repo
        self._scan_uc = scan_use_case
        self._process_uc = process_use_case

    def run(self, target_path: str, mode: str = "default") -> None:
        target_path = self._normalize_path(target_path)
        app_logger.info(f"🖥️ CLI Run Triggered | Mode: {mode} | Target: {target_path}")

        print(f"\n🚀 CodeContext AI: Запуск (Mode: {mode})...")
        print(f"🎯 Цель: {target_path}")

        if not self._validate(target_path):
            return

        # Загружаем настройки и пушим их в Store
        config = self._settings_repo.load()
        self._init_store(config, target_path)

        try:
            asyncio.run(self._pipeline(mode))
        except Exception as exc:
            app_logger.error(f"Critical CLI Error: {exc}\n{traceback.format_exc()}")
            print(f"\n🔥 Критическая ошибка: {exc}")
            traceback.print_exc()
        finally:
            self._keep_window_open()

    def _init_store(self, config: dict, target_path: str):
        """Загружает настройки в глобальный Redux Store."""
        extensions = config.get('extensions', PRESETS['Default']['ext'])
        if not extensions or not extensions.strip():
            extensions = PRESETS['Default']['ext']

        settings_dict = {
            'extensions': extensions,
            'ignored_paths': config.get('ignored_paths', PRESETS['Default']['ign']),
            'minify': config.get('cli_minify', True),
            'remove_comments': config.get('cli_remove_comments', True),
            'remove_secrets': config.get('cli_remove_secrets', True),
            'skeleton_mode': config.get('cli_skeleton_mode', False),
            'use_git': config.get('use_git', False),
            'use_gitignore': config.get('cli_use_gitignore', True),
            'include_tree': config.get('cli_include_tree', True),
            'include_dependencies': config.get('include_dependencies', False),
            'system_prompt': config.get('system_prompt', DEFAULT_SYSTEM_PROMPT),
            'output_format': config.get('cli_format', 'plain'),
            'template_path': config.get('template_path', ''),
            'enable_logging': config.get('enable_logging', True),
            'cli_minify': config.get('cli_minify', True),
            'cli_remove_comments': config.get('cli_remove_comments', True),
            'cli_remove_secrets': config.get('cli_remove_secrets', True),
            'cli_include_tree': config.get('cli_include_tree', True),
            'cli_skeleton_mode': config.get('cli_skeleton_mode', False),
            'cli_use_gitignore': config.get('cli_use_gitignore', True),
            'cli_format': config.get('cli_format', 'plain'),
            'python_interpreter': config.get('python_interpreter', '')
        }

        from ..actions.action_types import SETTINGS_LOADED, FOLDER_ADD
        self._dispatcher.dispatch(SETTINGS_LOADED, settings_dict)
        self._dispatcher.dispatch(FOLDER_ADD, target_path)

    async def _pipeline(self, mode: str) -> None:
        """Запускает scan → process через Use Cases, читая обновленный Store."""
        # 1. Сканируем
        await self._scan_uc.execute(self._store.state)

        # 2. Получаем обновленное состояние из Store
        current_state = self._store.state

        if not current_state.scanned_files_paths:
            app_logger.warning("CLI: Файлы не найдены.")
            print("⚠️ Файлы не найдены.")
            return

        # 3. Обрабатываем
        await self._process_uc.execute(current_state, target='clipboard')

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path:
            return ""
        return os.path.abspath(path.strip('"\''))

    @staticmethod
    def _validate(path: str) -> bool:
        if not os.path.exists(path):
            app_logger.error(f"CLI Error: Путь не существует {path}")
            print(f"❌ Ошибка: Путь не существует: {path}")
            CliController._keep_window_open()
            return False
        return True

    @staticmethod
    def _keep_window_open():
        print("\n(Окно закроется через 3 секунды...)")
        time.sleep(3)