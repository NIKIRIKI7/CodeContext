import os
import sys
import asyncio
import time
import traceback
import difflib
import json
import urllib.request
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

    def run(self, target_path: str, **kwargs) -> None:
        target_path = self._normalize_path(target_path)
        silent = kwargs.get('silent', False)
        
        if silent:
            sys.stdout = open(os.devnull, 'w')

        mode = kwargs.get('mode', 'default')
        app_logger.info(f"🖥️ CLI Run Triggered | Mode: {mode} | Target: {target_path}")
        
        print(f"\n🚀 CodeContext AI: Запуск (Mode: {mode})...")
        print(f"🎯 Цель: {target_path}")

        if not self._validate(target_path, silent):
            return

        config = self._settings_repo.load()
        self._init_store(config, target_path, kwargs)

        try:
            asyncio.run(self._pipeline(kwargs))
        except Exception as exc:
            app_logger.error(f"Critical CLI Error: {exc}\n{traceback.format_exc()}")
            print(f"\n🔥 Критическая ошибка: {exc}")
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

        print(f"\n🚀 CodeContext AI: Safety Patch Mode")
        print(f"🎯 Цель: {target_path}")
        print(f"📄 Файл патча: {patch_file}\n")

        if not self._validate(target_path, False):
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
            'use_git': config.get('use_git', False),
            'llm_model': config.get('llm_model', 'gpt-4o-mini')
        }

        from ..actions.action_types import SETTINGS_LOADED, FOLDER_ADD
        self._dispatcher.dispatch(SETTINGS_LOADED, settings_dict)
        self._dispatcher.dispatch(FOLDER_ADD, target_path)

    async def _fetch_pricing_data(self, model_query: str) -> dict:
        """Получает актуальные цены с OpenRouter API в отдельном потоке"""
        def fetch():
            try:
                url = "https://openrouter.ai/api/v1/models"
                req = urllib.request.Request(url, headers={"User-Agent": "CodeContextAI-App"})
                with urllib.request.urlopen(req, timeout=5) as response:
                    data = json.loads(response.read().decode('utf-8'))
                models = data.get('data', [])
                for m in models:
                    m_id = m.get('id', '').lower()
                    if m_id == model_query.lower() or m_id.split('/')[-1] == model_query.lower():
                        return m
                for m in models:
                    if model_query.lower() in m.get('id', '').lower():
                        return m
            except Exception as e:
                app_logger.warning(f"OpenRouter API error: {e}")
            return None

        model_info = await asyncio.to_thread(fetch)

        default_prices = {
            "gpt-4o-mini": 0.00000015,
            "gpt-4o": 0.0000025,
            "claude-3-5-sonnet": 0.000003,
            "gpt-4-turbo": 0.00001
        }

        if model_info:
            try:
                price = float(model_info.get('pricing', {}).get('prompt', 0))
                return {"name": model_info.get('name', model_query), "price": price}
            except (ValueError, TypeError):
                pass

        for k, v in default_prices.items():
            if k in model_query.lower():
                return {"name": f"{k} (offline)", "price": v}

        return {"name": model_query, "price": 0.0}

    async def _pipeline(self, kwargs: dict) -> None:
        await self._scan_uc.execute(self._store.state)
        current_state = self._store.state

        if not current_state.scanned_files_paths:
            app_logger.warning("CLI: Файлы не найдены.")
            print("⚠️ Файлы не найдены.")
            return

        if kwargs.get('dry_run', False):
            tokens = current_state.selected_tokens
            count = len(current_state.scanned_files_paths)
            model = current_state.settings.llm_model or "gpt-4o-mini"

            print("\n📊 ОЦЕНКА КОНТЕКСТА (DRY-RUN)")
            print("==================================")
            print(f"📁 Найдено файлов: {count}")
            print(f"⚖️ Примерный объем: {tokens} токенов")
            print("⏳ Получение актуальных цен из API...")

            price_data = await self._fetch_pricing_data(model)

            if price_data['price'] > 0:
                cost = tokens * price_data['price']
                print(f"💰 Оценка ({price_data['name']}): ~${cost:.6f} USD")
            else:
                print(f"💰 Оценка ({price_data['name']}): Бесплатно или локальная модель")

            print("==================================")
            return

        await self._process_uc.execute(current_state, target='clipboard')

        if kwargs.get('silent', False):
            self._trigger_native_notification(current_state.selected_tokens)

    @staticmethod
    def _trigger_native_notification(tokens: int):
        import platform
        try:
            sys_name = platform.system()
            msg = f"Проект скопирован! ({tokens} токенов)"
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
        if not path:
            return ""
        return os.path.abspath(path.strip('"\''))

    @staticmethod
    def _validate(path: str, silent: bool = False) -> bool:
        if not os.path.exists(path):
            app_logger.error(f"CLI Error: Путь не существует {path}")
            if not silent:
                print(f"❌ Ошибка: Путь не существует: {path}")
                CliController._keep_window_open()
            return False
        return True

    @staticmethod
    def _keep_window_open():
        print("\n(Окно закроется через 3 секунды...)")
        time.sleep(3)