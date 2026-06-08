import json
import os
from typing import Dict, Any
from ..utils.config import get_app_data_dir

class SettingsRepository:
    """Управление файлом настроек в безопасной директории ОС"""

    def __init__(self, filename: str = "user_settings.json"):
        self.filepath = os.path.join(get_app_data_dir(), filename)

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
