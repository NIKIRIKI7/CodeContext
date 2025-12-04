"""
Service to manage persistent user settings (JSON).
"""
import json
import os
from src.config import DEFAULT_SYSTEM_PROMPT, PRESETS

SETTINGS_FILE = "user_settings.json"

DEFAULT_SETTINGS = {
    "cli_exts": PRESETS['Default']['ext'],
    "cli_ign": PRESETS['Default']['ign'],
    "cli_minify": True,
    "cli_remove_comments": True,
    "cli_remove_secrets": True,
    "cli_include_tree": True,
    "cli_system_prompt": DEFAULT_SYSTEM_PROMPT,
    "cli_format": "markdown"
}


class SettingsManager:
    @staticmethod
    def load() -> dict:
        """Load settings from file or return defaults."""
        if not os.path.exists(SETTINGS_FILE):
            return DEFAULT_SETTINGS.copy()

        try:
            with open(SETTINGS_FILE, 'r', encoding='utf-8') as f:
                data = json.load(f)
                # Merge with defaults to ensure all keys exist (if version updated)
                settings = DEFAULT_SETTINGS.copy()
                settings.update(data)
                return settings
        except Exception:
            return DEFAULT_SETTINGS.copy()

    @staticmethod
    def save(settings: dict):
        """Save dictionary to settings file."""
        try:
            with open(SETTINGS_FILE, 'w', encoding='utf-8') as f:
                json.dump(settings, f, ensure_ascii=False, indent=2)
        except Exception as e:
            print(f"Error saving settings: {e}")