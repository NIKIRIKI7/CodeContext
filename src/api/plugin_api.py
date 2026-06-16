from types import SimpleNamespace
from typing import Any, Callable
from src.i18n import add_plugin_translations


class PluginAPI:
    def __init__(self, plugin_id: str, state, container, ui_registry=None):
        self._plugin_id = plugin_id
        self.state = state
        self.container = container

        if ui_registry is None:
            ui_registry = SimpleNamespace(sidebar_tabs={}, action_buttons={})
        self.ui = ui_registry

        self.ui.register_sidebar_tab = self._register_sidebar_tab
        self.ui.register_action_button = self._register_action_button

    # ponytail: fixed argument name to 'widget_factory' to match the actual plugin implementation
    def _register_sidebar_tab(self, tab_id: str, label: str, widget_factory: Callable[[], Any]) -> None:
        self.ui.sidebar_tabs[tab_id] = {"id": tab_id, "title": label, "label": label, "factory": widget_factory}

    def _register_action_button(self, action_id: str, label: str, callback: Callable) -> None:
        self.ui.action_buttons[action_id] = {"id": action_id, "label": label, "callback": callback}

    def add_translations(self, lang: str, data: dict) -> None:
        add_plugin_translations(lang, data)

    def add_log(self, message: str) -> None:
        self.state.add_log(f"[Plugin:{self._plugin_id}] {message}")


class IPlugin:
    id: str = ""
    name: str = ""
    version: str = ""

    def on_init(self, api: PluginAPI) -> None: pass
    def on_shutdown(self) -> None: pass
