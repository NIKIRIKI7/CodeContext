import json
import os
from typing import Dict, Any
from ..utils.config import get_app_data_dir


def load(filename: str = "user_settings.json") -> Dict[str, Any]:
    filepath = os.path.join(get_app_data_dir(), filename)
    if not os.path.exists(filepath):
        return {}
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading settings from {filepath}: {e}")
        return {}


def save(settings_dict: Dict[str, Any], filename: str = "user_settings.json"):
    filepath = os.path.join(get_app_data_dir(), filename)
    try:
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(settings_dict, f, ensure_ascii=False, indent=2)
    except Exception as e:
        print(f"Error saving settings to {filepath}: {e}")
