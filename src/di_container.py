import functools
from .store.state import AppState
from .services.plugin_manager import PluginManager
from .services.file_service import FileService
from .services.github_service import GitHubService
from .services.patch_service import PatchService
from .services import llm_checker_service
from .services.updater_service import UpdaterService
from .services.integration_service import IntegrationService
from .use_cases.scan_use_case import ScanWorkspaceUseCase
from .use_cases.process_use_case import ProcessWorkspaceUseCase
from .use_cases.github_use_case import GitHubUseCase
from .use_cases.settings_use_case import SettingsUseCase
from .use_cases.patch_use_case import PatchUseCase
from .use_cases.updater_use_case import UpdaterUseCase
from .controllers.main_controller import MainController
from .controllers.cli_controller import CliController

class DIContainer:
    def __init__(self):
        self.state = AppState()
        self.plugin_manager = PluginManager(self.state, self)

    @functools.cached_property
    def file_service(self):
        return FileService()

    @functools.cached_property
    def github_service(self):
        return GitHubService()

    @functools.cached_property
    def patch_service(self):
        return PatchService()

    @functools.cached_property
    def llm_checker(self):
        return llm_checker_service

    @functools.cached_property
    def updater_service(self):
        return UpdaterService()

    @functools.cached_property
    def integration_service(self):
        return IntegrationService()

    @functools.cached_property
    def scan_use_case(self):
        return ScanWorkspaceUseCase(self.state, self.file_service)

    @functools.cached_property
    def process_use_case(self):
        return ProcessWorkspaceUseCase()

    @functools.cached_property
    def github_use_case(self):
        return GitHubUseCase(self.state, self.github_service)

    @functools.cached_property
    def settings_use_case(self):
        return SettingsUseCase(self.state)

    @functools.cached_property
    def patch_use_case(self):
        return PatchUseCase(self.state, self.patch_service)

    @functools.cached_property
    def updater_use_case(self):
        return UpdaterUseCase(self.state, self.updater_service)

    @functools.cached_property
    def main_controller(self):
        return MainController(
            state=self.state,
            scan_use_case=self.scan_use_case,
            process_use_case=self.process_use_case,
            github_use_case=self.github_use_case,
            settings_use_case=self.settings_use_case,
            patch_use_case=self.patch_use_case,
            updater_use_case=self.updater_use_case,
            integration_service=self.integration_service,
            llm_checker=self.llm_checker,
            plugin_manager=self.plugin_manager,
        )

    @functools.cached_property
    def cli_controller(self):
        return CliController(
            state=self.state,
            scan_use_case=self.scan_use_case,
            process_use_case=self.process_use_case,
            patch_use_case=self.patch_use_case,
        )
