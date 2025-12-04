import json
import os
import sys
from typing import Dict, Any


class SettingsRepository:
    """Управление файлом настроек с защитой путей"""

    def __init__(self, filename: str = "user_settings.json"):
        # Определяем корневую директорию приложения
        if getattr(sys, 'frozen', False):
            # Если запущено как скомпилированный EXE
            base_dir = os.path.dirname(sys.executable)
        else:
            # Если запущено как скрипт, берем директорию main.py (entry point)
            # sys.argv[0] надежнее __file__ при запуске из других директорий
            base_dir = os.path.dirname(os.path.abspath(sys.argv[0]))

        self.filepath = os.path.join(base_dir, filename)

    def load(self) -> Dict[str, Any]:
        """Загрузка настроек из JSON"""
        if not os.path.exists(self.filepath):
            return {}
        try:
            with open(self.filepath, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error loading settings from {self.filepath}: {e}")
            return {}

    def save(self, settings_dict: Dict[str, Any]):
        """Сохранение настроек в JSON"""
        try:
            with open(self.filepath, 'w', encoding='utf-8') as f:
                json.dump(settings_dict, f, ensure_ascii=False, indent=2)
        except Exception as e:
            print(f"Error saving settings to {self.filepath}: {e}")