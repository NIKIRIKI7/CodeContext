import os
import sys
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
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT, PricingManager
from ..utils.logger import app_logger
from src.i18n import tr

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

    def run(self, target_path: str, **kwargs) -> None:
        target_path = self._normalize_path(target_path)

        silent = kwargs.get('silent', False)
        if silent:
            sys.stdout = open(os.devnull, 'w', encoding='utf-8')
        sys.stdout.reconfigure(encoding='utf-8')

        mode = kwargs.get('mode', 'default')
        app_logger.info(f"🖥️ CLI Run Triggered | Mode: {mode} | Target: {target_path}")
        print(tr("cli_controller.starting", mode=mode))
        print(tr("cli_controller.target", target_path=target_path))

        if not self._validate(target_path, silent):
            return

        config = self._settings_repo.load()
        self._init_store(config, target_path, kwargs)

        try:
            asyncio.run(self._pipeline(kwargs))
        except Exception as exc:
            app_logger.error(f"Critical CLI Error: {exc}\n{traceback.format_exc()}")
            print(tr("cli_controller.critical_error", exc=exc))
            if not silent:
                traceback.print_exc()
        finally:
            if not silent:
                self._keep_window_open()
            if silent:
                sys.stdout.close()
                sys.stdout = sys.__stdout__

    def run_patch(self, target_path: str, patch_file: str) -> None:
        """Интерактивный режим применения JSON-патчей с подтверждением."""
        target_path = self._normalize_path(target_path)
        patch_file = self._normalize_path(patch_file)

        print(tr("cli_controller.safety_patch_mode"))
        print(tr("cli_controller.patch_target", target_path=target_path))
        print(tr("cli_controller.patch_file", patch_file=patch_file))

        if not self._validate(target_path, False):
            return

        if not os.path.exists(patch_file):
            print(tr("cli_controller.patch_file_not_found", patch_file=patch_file))
            return

        with open(patch_file, 'r', encoding='utf-8') as f:
            patch_str = f.read()

        prepared_patches = self._patch_uc.prepare_json_patch(patch_str, [target_path])
        if not prepared_patches:
            print(tr("cli_controller.patches_invalid"))
            return

        patches_to_apply = []
        for p in prepared_patches:
            if not p['success']:
                print(tr("cli_controller.patch_skipped", file_target=p['file_target'], msg=p['msg']))
                continue

            print(tr("cli_controller.changes_for", file_target=p['file_target'], action=p.get('action', 'unknown').upper()))
            self._print_unified_diff(p['original_content'], p['patched_content'], p['file_target'])

            while True:
                choice = input(tr("cli_controller.apply_prompt")).strip().lower()
                if choice in ('y', 'yes', ''):
                    patches_to_apply.append(p)
                    break
                elif choice in ('n', 'no'):
                    print(tr("cli_controller.skipped"))
                    break
                elif choice in ('q', 'quit'):
                    print(tr("cli_controller.cancelled"))
                    self._apply_approved_patches(patches_to_apply)
                    return
                else:
                    print(tr("cli_controller.invalid_choice"))

        self._apply_approved_patches(patches_to_apply)

    def _apply_approved_patches(self, patches: list):
        if not patches:
            print(tr("cli_controller.no_patches"))
            return
 
        print(tr("cli_controller.applying_patches", count=len(patches)))
        applied, logs = self._patch_uc.apply_prepared(patches)
        for log in logs:
            print(log)
        print(tr("cli_controller.applied_count", applied=applied))

    def _print_unified_diff(self, original: str, patched: str, filename: str):
        GREEN = '\033[92m'
        RED = '\033[91m'
        CYAN = '\033[96m'
        RESET = '\033[0m'

        orig_lines = original.splitlines(keepends=True) if original else []
        patched_lines = patched.splitlines(keepends=True) if patched else []

        diff = list(difflib.unified_diff(
            orig_lines, patched_lines,
            fromfile=f"a/{filename}", tofile=f"b/{filename}", n=3
        ))

        if not diff:
            print(tr("cli_controller.no_changes"))
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

    def _init_store(self, config: dict, target_path: str, kwargs: dict):
        extensions = config.get('extensions', PRESETS['Default']['ext'])
        if not extensions or not extensions.strip():
            extensions = PRESETS['Default']['ext']

        minify = kwargs.get('minify') == 'true' if kwargs.get('minify') else config.get('minify', True)
        skeleton = kwargs.get('skeleton') == 'true' if kwargs.get('skeleton') else config.get('skeleton_mode', False)
        out_fmt = kwargs.get('format') if kwargs.get('format') else config.get('output_format', 'plain')

        settings_dict = {
            'extensions': extensions,
            'ignored_paths': config.get('ignored_paths', PRESETS['Default']['ign']),
            'minify': minify,
            'remove_comments': config.get('remove_comments', True),
            'remove_secrets': config.get('remove_secrets', True),
            'skeleton_mode': skeleton,
            'use_gitignore': config.get('use_gitignore', True),
            'include_tree': config.get('include_tree', True),
            'include_dependencies': config.get('include_dependencies', False),
            'output_format': out_fmt,
            'system_prompt': config.get('system_prompt', DEFAULT_SYSTEM_PROMPT),
            'template_path': config.get('template_path', ''),
            'enable_logging': config.get('enable_logging', True) and not kwargs.get('silent', False),
            'use_git': kwargs.get('git') if kwargs.get('git') else config.get('use_git', False),
            'git_base': kwargs.get('git_base', ''),
            'llm_model': config.get('llm_model', 'gpt-4o-mini')
        }

        from ..actions.action_types import SETTINGS_LOADED, FOLDER_ADD
        self._dispatcher.dispatch(SETTINGS_LOADED, settings_dict)
        self._dispatcher.dispatch(FOLDER_ADD, target_path)

    async def _pipeline(self, kwargs: dict) -> None:
        await self._scan_uc.execute(self._store.state)
        current_state = self._store.state

        if not current_state.scanned_files_paths:
            app_logger.warning("CLI: Файлы не найдены.")
            print(tr("cli_controller.no_files_found"))
            return

        limit = kwargs.get('fail_if_exceeds')
        if limit and current_state.selected_tokens > limit:
            import sys
            print(tr("cli_controller.token_limit_exceeded", tokens=current_state.selected_tokens, limit=limit), file=sys.stderr)
            sys.exit(1)

        if kwargs.get('dry_run', False):
            tokens = current_state.selected_tokens
            count = len(current_state.scanned_files_paths)
            model = current_state.settings.llm_model or "gpt-4o-mini"

            print(tr("cli_controller.dry_run_header"))
            print("==================================")
            print(tr("cli_controller.files_found", count=count))
            print(tr("cli_controller.estimated_volume", tokens=tokens))
            print(tr("cli_controller.fetching_prices"))

            def fetch_and_get():
                PricingManager.fetch_prices_sync()
                return PricingManager.get_price(model)

            price = await asyncio.to_thread(fetch_and_get)

            if price > 0:
                cost = tokens * price
                print(tr("cli_controller.cost_estimate", model=model, cost=cost))
            else:
                print(tr("cli_controller.free_or_local", model=model))
            print("==================================")
            return

        target = 'stdout' if kwargs.get('stdout') else 'clipboard'
        await self._process_uc.execute(current_state, target=target)

        if kwargs.get('silent', False) and not kwargs.get('stdout'):
            self._trigger_native_notification(current_state.selected_tokens)

    @staticmethod
    def _trigger_native_notification(tokens: int):
        import platform
        try:
            sys_name = platform.system()
            msg = tr("cli_controller.project_copied", tokens=tokens)
            if sys_name == "Windows":
                script = f'[reflection.assembly]::loadwithpartialname("System.Windows.Forms");[system.Windows.Forms.MessageBox]::show("{msg}", "CodeContext AI")'
                os.system(f'powershell -Command "{script}"')
            elif sys_name == "Linux":
                os.system(f'notify-send "CodeContext AI" "{msg}"')
            elif sys_name == "Darwin":
                os.system(f'osascript -e \'display notification "{msg}" with title "CodeContext AI"\'')
        except Exception:
            pass

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path: return ""
        return os.path.abspath(path.strip('"\''))

    @staticmethod
    def _validate(path: str, silent: bool = False) -> bool:
        if not os.path.exists(path):
            app_logger.error(f"CLI Error: Путь не существует {path}")
            if not silent:
                print(tr("cli_controller.path_not_found", path=path))
                CliController._keep_window_open()
            return False
        return True

    @staticmethod
    def _keep_window_open():
        print(tr("cli_controller.window_closing"))
        time.sleep(3)
