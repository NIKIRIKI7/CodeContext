import platform
from types import SimpleNamespace
from .store.state import AppState
from .data.file_system_repository import FileSystemRepository
from .data.settings_repository import SettingsRepository
from .api.plugin_api import PluginAPI
from .services.plugin_manager import PluginManager
from .services.file_service import FileService
from .services.github_service import GitHubService
from .services.patch_service import PatchService
from .services.tour_service import TourService
from .services.llm_checker_service import LlmCheckerService
from .services.updater_service import UpdaterService
from .services.strategies.integration_strategies import WindowsContextMenuStrategy, LinuxContextMenuStrategy, MacOSContextMenuStrategy
from .use_cases.scan_use_case import ScanWorkspaceUseCase
from .use_cases.process_use_case import ProcessWorkspaceUseCase
from .use_cases.github_use_case import GitHubUseCase
from .use_cases.settings_use_case import SettingsUseCase
from .use_cases.patch_use_case import PatchUseCase
from .use_cases.updater_use_case import UpdaterUseCase
from .controllers.main_controller import MainController
from .controllers.cli_controller import CliController


def _make_integration_strategy():
    system = platform.system()
    if system == "Windows": return WindowsContextMenuStrategy()
    if system == "Linux": return LinuxContextMenuStrategy()
    return MacOSContextMenuStrategy()


class DIContainer:
    def __init__(self):
        self.state = AppState()
        self.fs_repo = FileSystemRepository()
        self.settings_repo = SettingsRepository()
        self.ui_registry = SimpleNamespace(sidebar_tabs={}, action_buttons={})

        self.plugin_api = PluginAPI("core", self.state, self, self.ui_registry)
        self.plugin_manager = PluginManager(self.state, self, self.ui_registry)

    @property
    def file_service(self):
        return FileService(self.fs_repo)

    @property
    def github_service(self):
        return GitHubService()

    @property
    def patch_service(self):
        return PatchService()

    @property
    def tour_service(self):
        return TourService()

    @property
    def llm_checker(self):
        return LlmCheckerService()

    @property
    def updater_service(self):
        return UpdaterService()

    @property
    def integration_strategy(self):
        return _make_integration_strategy()

    @property
    def scan_use_case(self):
        return ScanWorkspaceUseCase(self.state, self.file_service, self.fs_repo)

    @property
    def process_use_case(self):
        return ProcessWorkspaceUseCase(self.fs_repo)

    @property
    def github_use_case(self):
        return GitHubUseCase(self.state, self.github_service)

    @property
    def settings_use_case(self):
        return SettingsUseCase(self.state, self.settings_repo, self.fs_repo)

    @property
    def patch_use_case(self):
        return PatchUseCase(self.state, self.patch_service)

    @property
    def updater_use_case(self):
        return UpdaterUseCase(self.state, self.updater_service)

    @property
    def main_controller(self):
        if not hasattr(self, '_main_controller'):
            self._main_controller = MainController(
                state=self.state,
                scan_use_case=self.scan_use_case,
                process_use_case=self.process_use_case,
                github_use_case=self.github_use_case,
                settings_use_case=self.settings_use_case,
                patch_use_case=self.patch_use_case,
                updater_use_case=self.updater_use_case,
                integration_strategy=self.integration_strategy,
                fs_repo=self.fs_repo,
                tour_service=self.tour_service,
                llm_checker=self.llm_checker,
                plugin_api=self.plugin_api,
                plugin_manager=self.plugin_manager
            )
        return self._main_controller

    @property
    def cli_controller(self):
        return CliController(
            state=self.state,
            settings_repo=self.settings_repo,
            scan_use_case=self.scan_use_case,
            process_use_case=self.process_use_case,
            patch_use_case=self.patch_use_case,
        )
