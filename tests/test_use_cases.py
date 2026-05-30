import pytest
from unittest.mock import MagicMock, AsyncMock

from src.use_cases.scan_use_case import ScanWorkspaceUseCase
from src.use_cases.process_use_case import ProcessWorkspaceUseCase
from src.use_cases.patch_use_case import PatchUseCase
from src.use_cases.github_use_case import GitHubUseCase
from src.use_cases.settings_use_case import SettingsUseCase
from src.actions.action_types import *
from src.store.state import AppState, AppSettings
from src.utils.config import PRESETS
from src.use_cases.settings_use_case import _DEFAULT_SETTINGS


# --- SCAN USE CASE ---

@pytest.mark.asyncio
async def test_scan_use_case_success(mock_dispatcher, mock_fs_repo):
    file_service = MagicMock()
    file_service.scan_folders_async = AsyncMock(return_value=["/path/to/main.py"])

    mock_fs_repo.get_git_status_async.return_value = {"/path/to/main.py": "modified"}

    uc = ScanWorkspaceUseCase(mock_dispatcher, file_service, mock_fs_repo)
    state = AppState(selected_folders=["/path/to"])

    await uc.execute(state)

    mock_dispatcher.dispatch.assert_any_call(UI_SET_LOADING, True)

    call_args = [call for call in mock_dispatcher.dispatch.call_args_list if call[0][0] == SCAN_SUCCESS]
    assert len(call_args) == 1
    payload = call_args[0][0][1]
    assert payload['paths'] == ["/path/to/main.py"]
    assert payload['metadata']["/path/to/main.py"]['git_status'] == "modified"


@pytest.mark.asyncio
async def test_scan_use_case_empty(mock_dispatcher, mock_fs_repo):
    file_service = MagicMock()
    file_service.scan_folders_async = AsyncMock(return_value=[])

    uc = ScanWorkspaceUseCase(mock_dispatcher, file_service, mock_fs_repo)
    await uc.execute(AppState())

    mock_dispatcher.dispatch.assert_any_call(SCAN_FAILURE, "Файлы не найдены")


# --- PROCESS USE CASE ---

@pytest.mark.asyncio
async def test_process_use_case(mock_dispatcher):
    process_service = MagicMock()
    process_service.read_files_async = AsyncMock(return_value=[
        {"path": "test.py", "content": "print('hello')", "ext": ".py"}
    ])

    dep_service = MagicMock()
    dep_service.resolve_dependencies = AsyncMock(return_value={})

    cleaner_service = MagicMock()
    cleaner_service.clean = MagicMock(return_value="print('hello')")

    skeleton_service = MagicMock()
    token_service = MagicMock()
    token_service.count_tokens = MagicMock(return_value=5)

    format_service = MagicMock()
    format_service.format_output = MagicMock(return_value="FORMATTED TEXT")

    output_service = MagicMock()

    uc = ProcessWorkspaceUseCase(
        mock_dispatcher, process_service, dep_service, cleaner_service,
        skeleton_service, token_service, format_service, output_service
    )

    state = AppState(scanned_files_paths=["test.py"])
    await uc.execute(state, target="clipboard")

    mock_dispatcher.dispatch.assert_any_call(WORKFLOW_STARTED, {'message': "Подготовка файлов...", 'progress': 0.1})
    mock_dispatcher.dispatch.assert_any_call(WORKFLOW_FINISHED, None)
    output_service.copy_to_clipboard.assert_called_once_with("FORMATTED TEXT")


# --- PATCH USE CASE ---

def test_patch_use_case_prepare(mock_dispatcher):
    patch_service = MagicMock()
    patch_service.prepare_patches.return_value = [{"success": True}]

    uc = PatchUseCase(mock_dispatcher, patch_service)

    md_json = """
    Sure, here is the code:
    ```json
    [{"action": "delete", "file": "test.py"}]
    ```
    """
    res = uc.prepare_json_patch(md_json, ["/base"])
    assert len(res) == 1
    patch_service.prepare_patches.assert_called_once()

    res_invalid = uc.prepare_json_patch("just normal text", ["/base"])
    assert res_invalid == []
    mock_dispatcher.dispatch.assert_any_call(UI_ADD_LOG, "❌ Ошибка: Не найдено валидных JSON-инструкций в тексте.")


def test_patch_use_case_apply(mock_dispatcher):
    patch_service = MagicMock()
    patch_service.apply_prepared.return_value = (1, ["Log msg"])

    uc = PatchUseCase(mock_dispatcher, patch_service)
    applied, logs = uc.apply_prepared([{"success": True}])

    assert applied == 1
    assert "Log msg" in logs
    mock_dispatcher.dispatch.assert_any_call(UI_ADD_LOG, "Log msg")


# --- GITHUB USE CASE ---

@pytest.mark.asyncio
async def test_github_use_case(mock_dispatcher):
    github_service = MagicMock()
    github_service.clone_repo_async = AsyncMock(return_value="/tmp/repo")

    uc = GitHubUseCase(mock_dispatcher, github_service)

    await uc.execute("https://github.com/user/repo")
    mock_dispatcher.dispatch.assert_any_call(UI_SET_LOADING, True)
    mock_dispatcher.dispatch.assert_any_call(GITHUB_CLONE_SUCCESS, {"path": "/tmp/repo", "is_temp": True})


@pytest.mark.asyncio
async def test_github_use_case_failure(mock_dispatcher):
    github_service = MagicMock()
    github_service.clone_repo_async = AsyncMock(side_effect=Exception("Git error"))

    uc = GitHubUseCase(mock_dispatcher, github_service)
    await uc.execute("https://github.com/user/repo")
    mock_dispatcher.dispatch.assert_any_call(GITHUB_CLONE_FAILURE, "Git error")


# --- SETTINGS USE CASE ---

def test_settings_use_case(mock_dispatcher, default_state, mock_settings_repo):
    store = MagicMock()
    store.state = default_state

    uc = SettingsUseCase(mock_dispatcher, store, mock_settings_repo)

    uc.load_initial()
    mock_dispatcher.dispatch.assert_called_with(SETTINGS_LOADED, _DEFAULT_SETTINGS)

    uc.update({"minify": False})
    mock_dispatcher.dispatch.assert_called_with(SETTINGS_UPDATE, {"minify": False})

    uc.apply_preset("Python Backend")
    preset = PRESETS["Python Backend"]
    mock_dispatcher.dispatch.assert_any_call(SETTINGS_UPDATE, {
        'extensions': preset['ext'],
        'ignored_paths': preset['ign']
    })

    uc.save()
    mock_settings_repo.save.assert_called_once()

    uc.reset()
    mock_settings_repo.save.assert_called()
