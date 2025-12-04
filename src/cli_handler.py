"""
CLI Handler for headless execution (Context Menu).
Reads configuration from user_settings.json
"""
import ctypes
import pyperclip
from src.services.file_scanner import FileScanner
from src.services.code_cleaner import CodeCleaner
from src.services.formatter_service import FormatterService
from src.services.settings_manager import SettingsManager

def run_headless(folder_path: str):
    """
    Runs the process using settings saved in user_settings.json.
    """
    try:
        # Load user settings
        cfg = SettingsManager.load()

        params = {
            'exts': cfg['cli_exts'].split(),
            'ign': {x.strip() for x in cfg['cli_ign'].split(',')},
            'minify': cfg['cli_minify'],
            'remove_comments': cfg['cli_remove_comments'],
            'remove_secrets': cfg['cli_remove_secrets'],
            'use_git': False, # CLI context menu implies scanning the specific folder
            'include_tree': cfg['cli_include_tree'],
            'system_prompt': cfg['cli_system_prompt'],
            'format': cfg['cli_format']
        }

        scanner = FileScanner(params['exts'], params['ign'])
        cleaner = CodeCleaner()

        files = scanner.scan([folder_path])
        if not files:
            ctypes.windll.user32.MessageBoxW(0, "Файлы кода не найдены (проверьте настройки расширений в приложении).", "CodeContext AI", 0x30)
            return

        file_entries = []
        for path in files:
            try:
                content = path.read_text(encoding='utf-8', errors='replace')
                cleaned = cleaner.process(content, path.suffix, params)
                file_entries.append({'path': str(path), 'content': cleaned})
            except:
                continue

        result = FormatterService.format(
            file_entries,
            params['format'],
            include_tree=params['include_tree'],
            system_prompt=params['system_prompt']
        )

        pyperclip.copy(result)

        msg = f"Обработано файлов: {len(file_entries)}.\nФормат: {params['format'].upper()}.\nСкопировано в буфер."
        ctypes.windll.user32.MessageBoxW(0, msg, "CodeContext AI: Готово", 0x40)

    except Exception as e:
        ctypes.windll.user32.MessageBoxW(0, f"Critical Error: {str(e)}", "CodeContext AI Error", 0x10)