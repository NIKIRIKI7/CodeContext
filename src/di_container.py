from .store.state import AppState
from .services.plugin_manager import PluginManager
from .services import llm_checker_service
from .controllers.main_controller import MainController
from .controllers.cli_controller import CliController


class DIContainer:
    def __init__(self):
        self.state = AppState()
        self.plugin_manager = PluginManager(self.state, self)
        self.main_controller = MainController(
            state=self.state,
            plugin_manager=self.plugin_manager,
            llm_checker=llm_checker_service,
        )
        self.cli_controller = CliController(
            state=self.state,
        )
