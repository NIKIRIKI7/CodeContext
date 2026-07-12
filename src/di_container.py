import functools
from .store.state import AppState
from .services.plugin_manager import PluginManager
from .services import llm_checker_service
from .controllers.main_controller import MainController
from .controllers.cli_controller import CliController


class DIContainer:
    def __init__(self):
        self.state = AppState()

    @functools.cached_property
    def plugin_manager(self):
        return PluginManager(self.state, self)

    @functools.cached_property
    def main_controller(self):
        return MainController(
            state=self.state,
            plugin_manager=self.plugin_manager,
            llm_checker=llm_checker_service,
        )

    @functools.cached_property
    def cli_controller(self):
        return CliController(
            state=self.state,
        )
