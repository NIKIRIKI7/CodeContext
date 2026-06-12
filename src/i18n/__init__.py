import json
import locale
import os
import sys

_translations: dict[str, str] = {}
_current_lang: str = "ru"
_loaded: set[str] = set()

_LANGUAGE_NAMES = {
    "ru": "Русский",
    "en": "English",
    "fr": "Français",
    "de": "Deutsch",
    "zh": "中文",
    "es": "Español",
    "it": "Italiano",
    "ar": "العربية",
    "pt": "Português",
    "ja": "日本語",
    "ko": "한국어",
    "hi": "हिन्दी",
    "tr": "Türkçe",
    "nl": "Nederlands",
    "pl": "Polski",
}

def _get_base_path() -> str:
    if getattr(sys, "frozen", False):
        return os.path.dirname(sys.executable)
    return os.path.dirname(os.path.abspath(__file__))

def _detect_system_lang() -> str:
    try:
        sys_lang, _ = locale.getdefaultlocale()
        if sys_lang:
            code = sys_lang.split("_")[0]
            if code in _LANGUAGE_NAMES:
                return code
    except Exception:
        pass
    return "en"

def load_translations(lang: str | None = None) -> None:
    global _current_lang
    if lang is None:
        lang = _detect_system_lang()
    _current_lang = lang if lang in _LANGUAGE_NAMES else "en"
    path = os.path.join(_get_base_path(), f"{_current_lang}.json")
    if not os.path.exists(path):
        path = os.path.join(_get_base_path(), "en.json")
        if not os.path.exists(path):
            _translations.clear()
            return
    if path in _loaded:
        return
    with open(path, encoding="utf-8-sig") as f:
        _translations.update(json.load(f))
    _loaded.add(path)

def tr(key: str, default: str = None, **kwargs) -> str:
    if not _translations:
        load_translations()
    text = _translations.get(key, default if default is not None else key)
    if kwargs:
        try:
            return text.format(**kwargs)
        except KeyError:
            return text
    return text

def current_lang() -> str:
    return _current_lang

def available_languages() -> dict[str, str]:
    return dict(_LANGUAGE_NAMES)

def set_language(lang: str) -> None:
    global _loaded
    _loaded.clear()
    _translations.clear()
    load_translations(lang)
