import pytest
from unittest.mock import MagicMock, AsyncMock
from src.store.state import AppState, AppSettings
from src.actions.dispatcher import Dispatcher
from src.data.file_system_repository import FileSystemRepository
from src.data.settings_repository import SettingsRepository


@pytest.fixture
def mock_dispatcher():
    return MagicMock(spec=Dispatcher)


@pytest.fixture
def mock_fs_repo():
    repo = MagicMock(spec=FileSystemRepository)
    repo.read_file_async = AsyncMock()
    repo.walk_directory_async = AsyncMock()
    repo.get_git_changed_files_async = AsyncMock()
    repo.get_git_status_async = AsyncMock()
    repo.delete_directory_async = AsyncMock()
    return repo


@pytest.fixture
def mock_settings_repo():
    repo = MagicMock(spec=SettingsRepository)
    repo.load = MagicMock(return_value={})
    repo.save = MagicMock()
    return repo


@pytest.fixture
def default_state():
    state = AppState()
    state.settings = AppSettings()
    return state


@pytest.fixture
def dummy_options():
    class Options:
        remove_comments = True
        remove_secrets = True
        minify = True
        skeleton_mode = False
    return Options()
