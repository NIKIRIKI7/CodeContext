import abc
from typing import Any, Callable, Dict


class UIRegistry:
    def __init__(self):
        self.sidebar_tabs: Dict[str, dict] = {}
        self.action_buttons: Dict[str, dict] = {}

    def register_sidebar_tab(self, tab_id: str, label: str, widget_factory: Callable[[], Any]) -> None:
        self.sidebar_tabs[tab_id] = {
            "id": tab_id,
            "title": label,
            "label": label,
            "factory": widget_factory,
        }

    def register_action_button(self, action_id: str, label: str, callback: Callable) -> None:
        self.action_buttons[action_id] = {
            "id": action_id,
            "label": label,
            "callback": callback,
        }


class PluginAPI:
    def __init__(self, plugin_id: str, store, dispatcher, container, ui_registry: UIRegistry):
        self._plugin_id = plugin_id
        self._store = store
        self._dispatcher = dispatcher
        self._container = container
        self.ui = ui_registry

    @property
    def store(self):
        return self._store

    @property
    def dispatcher(self):
        return self._dispatcher

    @property
    def container(self):
        return self._container

    def add_translations(self, lang: str, data: dict) -> None:
        from src.i18n import add_plugin_translations
        add_plugin_translations(lang, data)

    def add_log(self, message: str) -> None:
        from src.actions.action_types import UI_ADD_LOG
        self._dispatcher.dispatch(UI_ADD_LOG, f"[Plugin:{self._plugin_id}] {message}")


class IPlugin(abc.ABC):
    id: str = ""
    name: str = ""
    version: str = ""

    @abc.abstractmethod
    def on_init(self, api: PluginAPI) -> None:
        pass

    def on_shutdown(self) -> None:
        pass
