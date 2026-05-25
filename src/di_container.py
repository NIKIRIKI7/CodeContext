import platform
from .store.store import Store
from .actions.dispatcher import Dispatcher
from .data.file_system_repository import FileSystemRepository
from .data.settings_repository import SettingsRepository
from .services.cleaner_service import CleanerService
from .services.dependency_service import DependencyService
from .services.file_service import FileService
from .services.formatting_service import FormattingService
from .services.github_service import GitHubService
from .services.import_resolution_service import ImportResolutionService
from .services.integration_service import IntegrationService
from .services.output_service import OutputService
from .services.processing_service import ProcessingService
from .services.skeleton_service import SkeletonService
from .services.token_service import TokenService
from .services.patch_service import PatchService
from .services.tour_service import TourService
from .services.llm_checker_service import LlmCheckerService  # НОВЫЙ ИМПОРТ

from .services.strategies.integration_strategies import (
    WindowsContextMenuStrategy,
    LinuxContextMenuStrategy,
    MacOSContextMenuStrategy,
)
from .use_cases.scan_use_case import ScanWorkspaceUseCase
from .use_cases.process_use_case import ProcessWorkspaceUseCase
from .use_cases.github_use_case import GitHubUseCase
from .use_cases.settings_use_case import SettingsUseCase
from .use_cases.patch_use_case import PatchUseCase
from .controllers.main_controller import MainController
from .controllers.cli_controller import CliController


def _make_integration_strategy():
    system = platform.system()
    if system == "Windows":
        return WindowsContextMenuStrategy()
    if system == "Linux":
        return LinuxContextMenuStrategy()
    return MacOSContextMenuStrategy()


class DIContainer:
    """Граф зависимостей приложения."""

    def __init__(self):
        self.store = Store()
        self.dispatcher = Dispatcher(self.store)

        self.fs_repo = FileSystemRepository()
        self.settings_repo = SettingsRepository()

        self.file_service = FileService(self.fs_repo)
        self.process_service = ProcessingService(self.fs_repo)
        self.cleaner_service = CleanerService()
        self.skeleton_service = SkeletonService()
        self.token_service = TokenService()
        self.format_service = FormattingService()
        self.output_service = OutputService()
        self.dependency_service = DependencyService()
        self.import_resolution_service = ImportResolutionService()
        self.github_service = GitHubService()
        self.patch_service = PatchService()
        self.tour_service = TourService()
        self.llm_checker = LlmCheckerService()  # НОВЫЙ СЕРВИС

        self.integration_service = IntegrationService(
            strategy=_make_integration_strategy()
        )

        self.scan_use_case = ScanWorkspaceUseCase(
            dispatcher=self.dispatcher,
            file_service=self.file_service,
            fs_repo=self.fs_repo
        )

        self.process_use_case = ProcessWorkspaceUseCase(
            dispatcher=self.dispatcher,
            process_service=self.process_service,
            dependency_service=self.dependency_service,
            cleaner_service=self.cleaner_service,
            skeleton_service=self.skeleton_service,
            token_service=self.token_service,
            format_service=self.format_service,
            output_service=self.output_service,
        )

        self.github_use_case = GitHubUseCase(
            dispatcher=self.dispatcher,
            github_service=self.github_service,
        )

        self.settings_use_case = SettingsUseCase(
            dispatcher=self.dispatcher,
            store=self.store,
            settings_repo=self.settings_repo,
        )

        self.patch_use_case = PatchUseCase(
            dispatcher=self.dispatcher,
            patch_service=self.patch_service
        )

        self.main_controller = MainController(
            store=self.store,
            dispatcher=self.dispatcher,
            scan_use_case=self.scan_use_case,
            process_use_case=self.process_use_case,
            github_use_case=self.github_use_case,
            settings_use_case=self.settings_use_case,
            patch_use_case=self.patch_use_case,
            integration_service=self.integration_service,
            fs_repo=self.fs_repo,
            tour_service=self.tour_service,
            llm_checker=self.llm_checker  # ПРОБРОС
        )

        self.cli_controller = CliController(
            store=self.store,
            dispatcher=self.dispatcher,
            settings_repo=self.settings_repo,
            scan_use_case=self.scan_use_case,
            process_use_case=self.process_use_case,
        )