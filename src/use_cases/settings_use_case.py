import json
import os
from pathlib import Path
from typing import Optional
from src.i18n import tr

from ..data.settings_repository import load as load_settings, save as save_settings
from ..store.state import AppState, AppSettings
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT

_DEFAULT_SETTINGS = {
    'extensions': PRESETS['Default']['ext'],
    'ignored_paths': PRESETS['Default']['ign'],
    'system_prompt': DEFAULT_SYSTEM_PROMPT,
    'minify': True,
    'remove_comments': True,
    'remove_secrets': True,
    'include_tree': True,
    'include_dependencies': False,
    'skeleton_mode': False,
    'use_git': False,
    'use_gitignore': True,
    'enable_logging': True,
    'output_format': 'markdown',
    'template_path': '',
    'python_interpreter': '',
    'deduplicate': False,
    'save_checkpoints': False,
    'auto_watch': False,
    'prioritize_entry_files': True,
    'preserve_docstrings': False,
    'preserve_imports': False,
    'aggressive_minify': False,
}


def _apply_language(lang: str):
    if not lang:
        from src.i18n import load_translations
        load_translations()
    else:
        from src.i18n import set_language
        set_language(lang)


def load_initial(state: AppState) -> None:
    data = load_settings()
    if not data:
        data = _DEFAULT_SETTINGS.copy()
    _merge_settings(state, data)
    _apply_language(data.get('language', ''))


def update_settings(state: AppState, settings_data: dict) -> None:
    _merge_settings(state, settings_data)


def save(state: AppState) -> None:
    save_settings(state.settings.__dict__)


def reset(state: AppState) -> None:
    _merge_settings(state, _DEFAULT_SETTINGS.copy())
    save_settings(_DEFAULT_SETTINGS)
    state.add_log(tr("settings_use_case.reset"))


def _merge_settings(state: AppState, data: dict):
    from dataclasses import replace
    valid_keys = set(AppSettings.__dataclass_fields__.keys())
    filtered = {k: v for k, v in data.items() if k in valid_keys}
    state.settings = replace(state.settings, **filtered)
    state.notify()


def load_local_config(state: AppState, folder_path: str) -> None:
    current_dir = Path(folder_path).resolve()
    for _ in range(5):
        for filename in [".codecontextrc.json", ".codecontextrc"]:
            cfg_path = current_dir / filename
            if cfg_path.exists():
                try:
                    local_settings = json.loads(cfg_path.read_text(encoding='utf-8'))
                    if not local_settings.get('extensions', "").strip():
                        local_settings.pop('extensions', None)
                    if not local_settings.get('ignored_paths', "").strip():
                        local_settings.pop('ignored_paths', None)
                    _merge_settings(state, local_settings)
                    state.add_log(tr("settings_use_case.local_config_applied", path=cfg_path.name))
                    return
                except Exception as exc:
                    state.add_log(tr("settings_use_case.config_read_error", error=exc))
                    return
        parent = current_dir.parent
        if parent == current_dir:
            break
        current_dir = parent


def save_local_config(state: AppState, folder_path: str) -> None:
    if not folder_path or not os.path.exists(folder_path):
        state.add_log(tr("settings_use_case.no_folder_selected"))
        return
    config_path = os.path.join(folder_path, ".codecontextrc.json")
    try:
        data = state.settings.__dict__.copy()
        for key in ['recent_workspaces', 'python_interpreter', 'custom_presets', 'custom_prompt_presets', 'llm_api_key']:
            data.pop(key, None)
        with open(config_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=4)
        state.add_log(tr("settings_use_case.config_saved", path=config_path))
    except Exception as exc:
        state.add_log(tr("settings_use_case.config_save_error", error=exc))


def apply_preset(state: AppState, preset_name: str) -> None:
    preset = PRESETS.get(preset_name)
    if preset:
        _merge_settings(state, {
            'extensions': preset['ext'],
            'ignored_paths': preset['ign'],
        })


def save_workspace(state: AppState, path: str) -> None:
    data = {
        'folders': state.selected_folders,
        'settings': state.settings.__dict__,
    }
    try:
        with open(path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=4)
        state.add_log(tr("settings_use_case.workspace_saved", path=path))
    except OSError as exc:
        state.add_log(tr("settings_use_case.workspace_save_error", error=exc))


def load_workspace(state: AppState, path: str) -> Optional[dict]:
    try:
        with open(path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        state.selected_folders = data.get('folders', [])
        _merge_settings(state, data.get('settings', {}))
        state.temp_folders = []
        state.add_log(tr("settings_use_case.workspace_loaded", path=path))
        return data
    except Exception as exc:
        state.add_log(tr("settings_use_case.workspace_load_error", error=exc))
        return None
