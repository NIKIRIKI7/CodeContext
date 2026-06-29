import os
import sys
import asyncio
import traceback
import difflib

from ..store.state import AppState
from ..data.settings_repository import load as load_settings
from ..use_cases import scan_use_case, process_use_case, patch_use_case
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT, PricingManager
from ..utils.logger import app_logger
from src.i18n import tr

class CliController:
    def __init__(
        self,
        state: AppState,
    ):
        self.state = state

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

        config = load_settings() or {}
        self._init_state(config, target_path, kwargs)

        try:
            asyncio.run(self._pipeline(kwargs))
        except Exception as exc:
            app_logger.error(f"Critical CLI Error: {exc}\n{traceback.format_exc()}")
            print(tr("cli_controller.critical_error", exc=exc))
            if not silent: traceback.print_exc()
        finally:
            if not silent: self._keep_window_open()
            if silent:
                sys.stdout.close()
                sys.stdout = sys.__stdout__

    def run_patch(self, target_path: str, patch_file: str) -> None:
        target_path = self._normalize_path(target_path)
        patch_file = self._normalize_path(patch_file)

        print(tr("cli_controller.safety_patch_mode"))
        print(tr("cli_controller.patch_target", target_path=target_path))
        print(tr("cli_controller.patch_file", patch_file=patch_file))

        if not self._validate(target_path, False): return
        if not os.path.exists(patch_file):
            print(tr("cli_controller.patch_file_not_found", patch_file=patch_file))
            return

        with open(patch_file, 'r', encoding='utf-8') as f:
            patch_str = f.read()

        self.state.selected_folders.append(target_path)
        prepared_patches = patch_use_case.prepare_json_patch(self.state, patch_str, [target_path])
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
        applied, logs = patch_use_case.apply_patches(self.state, patches)
        for log in logs: print(log)
        print(tr("cli_controller.applied_count", applied=applied))

    def _print_unified_diff(self, original: str, patched: str, filename: str):
        GREEN, RED, CYAN, RESET = '\033[92m', '\033[91m', '\033[96m', '\033[0m'
        orig_lines = original.splitlines(keepends=True) if original else []
        patched_lines = patched.splitlines(keepends=True) if patched else []
        diff = list(difflib.unified_diff(orig_lines, patched_lines, fromfile=f"a/{filename}", tofile=f"b/{filename}", n=3))
        if not diff:
            print(tr("cli_controller.no_changes"))
            return
        for line in diff:
            if line.startswith('+') and not line.startswith('+++'): print(f"{GREEN}{line.rstrip()}{RESET}")
            elif line.startswith('-') and not line.startswith('---'): print(f"{RED}{line.rstrip()}{RESET}")
            elif line.startswith('@@'): print(f"{CYAN}{line.rstrip()}{RESET}")
            else: print(line.rstrip())

    def _init_state(self, config: dict, target_path: str, kwargs: dict):
        settings = self.state.settings
        extensions = config.get('extensions', PRESETS['Default']['ext'])
        if not extensions or not extensions.strip(): extensions = PRESETS['Default']['ext']

        settings.extensions = extensions
        settings.ignored_paths = config.get('ignored_paths', PRESETS['Default']['ign'])
        settings.minify = kwargs.get('minify') == 'true' if kwargs.get('minify') else config.get('minify', True)
        settings.remove_comments = config.get('remove_comments', True)
        settings.remove_secrets = config.get('remove_secrets', True)
        settings.skeleton_mode = kwargs.get('skeleton') == 'true' if kwargs.get('skeleton') else config.get('skeleton_mode', False)
        settings.use_gitignore = config.get('use_gitignore', True)
        settings.include_tree = config.get('include_tree', True)
        settings.include_dependencies = config.get('include_dependencies', False)
        settings.output_format = kwargs.get('format') if kwargs.get('format') else config.get('output_format', 'plain')
        settings.system_prompt = config.get('system_prompt', DEFAULT_SYSTEM_PROMPT)
        settings.template_path = config.get('template_path', '')
        settings.enable_logging = config.get('enable_logging', True) and not kwargs.get('silent', False)
        settings.use_git = kwargs.get('git') if kwargs.get('git') else config.get('use_git', False)
        settings.git_base = kwargs.get('git_base', '')
        settings.llm_model = config.get('llm_model', 'gpt-4o-mini')

        if target_path not in self.state.selected_folders:
            self.state.selected_folders.append(target_path)

    async def _pipeline(self, kwargs: dict) -> None:
        await scan_use_case.scan_workspace(self.state)

        if not self.state.scanned_files_paths:
            app_logger.warning("CLI: Файлы не найдены.")
            print(tr("cli_controller.no_files_found"))
            return

        limit = kwargs.get('fail_if_exceeds')
        if limit and self.state.selected_tokens > limit:
            print(tr("cli_controller.token_limit_exceeded", tokens=self.state.selected_tokens, limit=limit), file=sys.stderr)
            sys.exit(1)

        if kwargs.get('dry_run', False):
            tokens = self.state.selected_tokens
            count = len(self.state.scanned_files_paths)
            model = self.state.settings.llm_model or "gpt-4o-mini"
            print(tr("cli_controller.dry_run_header"))
            print("==================================")
            print(tr("cli_controller.files_found", count=count))
            print(tr("cli_controller.estimated_volume", tokens=tokens))
            print(tr("cli_controller.fetching_prices"))

            def fetch_and_get():
                PricingManager.fetch_prices_sync()
                return PricingManager.get_price(model)
            price = await asyncio.to_thread(fetch_and_get)

            if price > 0: print(tr("cli_controller.cost_estimate", model=model, cost=tokens * price))
            else: print(tr("cli_controller.free_or_local", model=model))
            print("==================================")
            return

        target = 'stdout' if kwargs.get('stdout') else 'clipboard'
        await process_use_case.process_workspace(self.state, target)

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path: return ""
        return os.path.abspath(path.strip('"\''))

    @staticmethod
    def _validate(path: str, silent: bool = False) -> bool:
        if not os.path.exists(path):
            app_logger.error(f"CLI Error: Путь не существует {path}")
            if not silent: print(tr("cli_controller.path_not_found", path=path))
            CliController._keep_window_open()
            return False
        return True

    @staticmethod
    def _keep_window_open():
        if sys.stdin is not None and sys.stdin.isatty() and sys.stdout.isatty():
            input(tr("cli_controller.window_closing") + " ")
