import json
import locale
import os
import sys

_translations: dict[str, str] = {}
_plugin_translations: dict[str, dict[str, str]] = {}
_current_lang: str = "en"

_LANGUAGE_NAMES = {
    "ru": "Русский", "en": "English", "fr": "Français", "de": "Deutsch",
    "zh": "中文", "es": "Español", "it": "Italiano", "ar": "العربية",
    "pt": "Português", "ja": "日本語", "ko": "한국어", "hi": "हिन्दी",
    "tr": "Türkçe", "nl": "Nederlands", "pl": "Polski",
}

def load_translations(lang: str = None) -> None:
    global _current_lang
    if not lang:
        try: lang = locale.getdefaultlocale()[0][:2]
        except Exception: lang = "en"
    _current_lang = lang if lang in _LANGUAGE_NAMES else "en"
    
    base = os.path.dirname(sys.executable) if getattr(sys, "frozen", False) else os.path.dirname(__file__)
    path = os.path.join(base, f"{_current_lang}.json")
    if not os.path.exists(path): path = os.path.join(base, "en.json")
    
    _translations.clear()
    try:
        with open(path, encoding="utf-8-sig") as f: _translations.update(json.load(f))
    except Exception: pass

def add_plugin_translations(lang: str, data: dict) -> None:
    _plugin_translations.setdefault(lang, {}).update(data)

def tr(key: str, default: str = None, **kwargs) -> str:
    if not _translations: load_translations()
    text = _translations.get(key)
    if text is None:
        text = _plugin_translations.get(_current_lang, {}).get(key, default if default is not None else key)
    try: return text.format(**kwargs) if kwargs else text
    except KeyError: return text

def current_lang() -> str: return _current_lang
def available_languages() -> dict[str, str]: return _LANGUAGE_NAMES.copy()
def set_language(lang: str) -> None: load_translations(lang)
