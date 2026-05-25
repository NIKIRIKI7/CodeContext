import os
import asyncio
import time
import traceback
import difflib

from ..store.store import Store
from ..actions.dispatcher import Dispatcher
from ..data.settings_repository import SettingsRepository
from ..use_cases.scan_use_case import ScanWorkspaceUseCase
from ..use_cases.process_use_case import ProcessWorkspaceUseCase
from ..use_cases.patch_use_case import PatchUseCase
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
            patch_use_case: PatchUseCase = None,
    ):
        self._store = store
        self._dispatcher = dispatcher
        self._settings_repo = settings_repo
        self._scan_uc = scan_use_case
        self._process_uc = process_use_case
        self._patch_uc = patch_use_case

    def run(self, target_path: str, mode: str = "default") -> None:
        target_path = self._normalize_path(target_path)
        app_logger.info(f"🖥️ CLI Run Triggered | Mode: {mode} | Target: {target_path}")
        print(f"\n🚀 CodeContext AI: Запуск (Mode: {mode})...")
        print(f"🎯 Цель: {target_path}")

        if not self._validate(target_path):
            return

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

    def run_patch(self, target_path: str, patch_file: str) -> None:
        """Интерактивный режим применения JSON-патчей с подтверждением."""
        target_path = self._normalize_path(target_path)
        patch_file = self._normalize_path(patch_file)

        print(f"\n🚀 CodeContext AI: Safety Patch Mode")
        print(f"🎯 Цель: {target_path}")
        print(f"📄 Файл патча: {patch_file}\n")

        if not self._validate(target_path):
            return

        if not os.path.exists(patch_file):
            print(f"❌ Ошибка: Файл патча не найден: {patch_file}")
            return

        with open(patch_file, 'r', encoding='utf-8') as f:
            patch_str = f.read()

        prepared_patches = self._patch_uc.prepare_json_patch(patch_str, [target_path])

        if not prepared_patches:
            print("⚠️ Патчи не найдены или JSON не валиден.")
            return

        patches_to_apply = []
        for p in prepared_patches:
            if not p['success']:
                print(f"❌ Пропущен ({p['file_target']}): {p['msg']}")
                continue

            print(f"\n=== Изменения для: {p['file_target']} [{p.get('action', 'unknown').upper()}] ===")
            self._print_unified_diff(p['original_content'], p['patched_content'], p['file_target'])

            while True:
                choice = input("\nПрименить эти изменения? [Y/n/q]: ").strip().lower()
                if choice in ('y', 'yes', ''):
                    patches_to_apply.append(p)
                    break
                elif choice in ('n', 'no'):
                    print("⏭️ Пропущено.")
                    break
                elif choice in ('q', 'quit'):
                    print("🛑 Отменено пользователем.")
                    self._apply_approved_patches(patches_to_apply)
                    return
                else:
                    print("Пожалуйста, введите Y, n или q.")

        self._apply_approved_patches(patches_to_apply)

    def _apply_approved_patches(self, patches: list):
        if not patches:
            print("\nНет патчей для применения.")
            return

        print(f"\nПрименение {len(patches)} патчей...")
        applied, logs = self._patch_uc.apply_prepared(patches)
        for log in logs:
            print(log)
        print(f"✅ Успешно применено: {applied}")

    def _print_unified_diff(self, original: str, patched: str, filename: str):
        """Отрисовывает цветной Diff прямо в терминале."""
        GREEN = '\033[92m'
        RED = '\033[91m'
        CYAN = '\033[96m'
        RESET = '\033[0m'

        orig_lines = original.splitlines(keepends=True) if original else []
        patched_lines = patched.splitlines(keepends=True) if patched else []

        diff = list(difflib.unified_diff(
            orig_lines,
            patched_lines,
            fromfile=f"a/{filename}",
            tofile=f"b/{filename}",
            n=3
        ))

        if not diff:
            print("Без изменений (содержимое идентично).")
            return

        for line in diff:
            if line.startswith('+') and not line.startswith('+++'):
                print(f"{GREEN}{line.rstrip()}{RESET}")
            elif line.startswith('-') and not line.startswith('---'):
                print(f"{RED}{line.rstrip()}{RESET}")
            elif line.startswith('@@'):
                print(f"{CYAN}{line.rstrip()}{RESET}")
            else:
                print(line.rstrip())

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
        await self._scan_uc.execute(self._store.state)
        current_state = self._store.state
        if not current_state.scanned_files_paths:
            app_logger.warning("CLI: Файлы не найдены.")
            print("⚠️ Файлы не найдены.")
            return

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