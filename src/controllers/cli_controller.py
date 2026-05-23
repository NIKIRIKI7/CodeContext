"""
CliController — тонкий CLI-контроллер.

Исправлено:
- Убрано прямое создание 10 сервисов (DIP нарушение).
- Логика pipeline делегирована Use Cases (SRP).
- Контроллер принимает зависимости через конструктор.
"""

import os
import asyncio
import time

from ..data.settings_repository import SettingsRepository
from ..use_cases.scan_use_case import ScanWorkspaceUseCase
from ..use_cases.process_use_case import ProcessWorkspaceUseCase
from ..store.state import AppState, AppSettings
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT


class CliController:
    """
    CLI-точка входа. Создаёт минимальное состояние и вызывает Use Cases.
    Не содержит бизнес-логики.
    """

    def __init__(
        self,
        settings_repo: SettingsRepository,
        scan_use_case: ScanWorkspaceUseCase,
        process_use_case: ProcessWorkspaceUseCase,
    ):
        self._settings_repo = settings_repo
        self._scan_uc = scan_use_case
        self._process_uc = process_use_case

    def run(self, target_path: str, mode: str = "default") -> None:
        target_path = self._normalize_path(target_path)
        print(f"\n🚀 CodeContext AI: Запуск (Mode: {mode})...")
        print(f"🎯 Цель: {target_path}")

        if not self._validate(target_path):
            return

        config = self._settings_repo.load()
        state = self._build_state(config, target_path)

        try:
            asyncio.run(self._pipeline(state, mode))
        except Exception as exc:
            print(f"\n🔥 Критическая ошибка: {exc}")
            import traceback
            traceback.print_exc()
        finally:
            self._keep_window_open()

    # ------------------------------------------------------------------
    # Private
    # ------------------------------------------------------------------

    async def _pipeline(self, state: AppState, mode: str) -> None:
        """Запускает scan → process через Use Cases."""
        # Scan
        await self._scan_uc.execute(state)

        if not state.scanned_files_paths:
            print("⚠️ Файлы не найдены.")
            return

        # Обновляем state после сканирования
        # (в CLI нет Store, поэтому читаем напрямую из scan_uc результата)
        # NOTE: для полноценного CLI без Store нужен отдельный CLI-dispatcher
        # с накапливаемым состоянием. Здесь используем упрощённую схему.
        await self._process_uc.execute(state, target='clipboard')

    @staticmethod
    def _build_state(config: dict, target_path: str) -> AppState:
        """Строит AppState из конфига и пути."""
        extensions = config.get('extensions', PRESETS['Default']['ext'])
        if not extensions or not extensions.strip():
            extensions = PRESETS['Default']['ext']

        settings = AppSettings(
            extensions=extensions,
            ignored_paths=config.get('ignored_paths', PRESETS['Default']['ign']),
            minify=config.get('cli_minify', True),
            remove_comments=config.get('cli_remove_comments', True),
            remove_secrets=config.get('cli_remove_secrets', True),
            skeleton_mode=config.get('cli_skeleton_mode', False),
            use_git=config.get('use_git', False),
            use_gitignore=config.get('cli_use_gitignore', True),
            include_tree=config.get('cli_include_tree', True),
            include_dependencies=config.get('include_dependencies', False),
            system_prompt=config.get('system_prompt', DEFAULT_SYSTEM_PROMPT),
            output_format=config.get('cli_format', 'plain'),
            template_path=config.get('template_path', ''),
        )

        state = AppState(settings=settings)
        state.selected_folders = [target_path]
        return state

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path:
            return ""
        return os.path.abspath(path.strip('"\''))

    @staticmethod
    def _validate(path: str) -> bool:
        if not os.path.exists(path):
            print(f"❌ Ошибка: Путь не существует: {path}")
            CliController._keep_window_open()
            return False
        return True

    @staticmethod
    def _keep_window_open():
        print("\n(Окно закроется через 3 секунды...)")
        time.sleep(3)