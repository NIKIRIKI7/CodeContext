import pytest
from unittest.mock import MagicMock, AsyncMock, patch
from src.controllers.main_controller import MainController
from src.controllers.cli_controller import CliController
from src.store.store import Store
from src.actions.dispatcher import Dispatcher
from src.utils.async_runtime import AsyncRuntime


def test_main_controller():
    store = Store()
    dispatcher = Dispatcher(store)
    scan_uc = MagicMock()
    process_uc = MagicMock()
    github_uc = MagicMock()
    settings_uc = MagicMock()
    patch_uc = MagicMock()
    integration_srv = MagicMock()
    fs_repo = MagicMock()
    tour_srv = MagicMock()
    llm_srv = MagicMock()
    format_srv = MagicMock()
    output_srv = MagicMock()

    ctrl = MainController(
        store, dispatcher, scan_uc, process_uc, github_uc, settings_uc,
        patch_uc, integration_srv, fs_repo, tour_srv, llm_srv, format_srv, output_srv
    )

    ctrl.load_initial_settings()
    settings_uc.load_initial.assert_called_once()

    ctrl.update_settings({"minify": False})
    settings_uc.update.assert_called_once()

    ctrl.save_settings()
    ctrl.reset_settings()
    ctrl.apply_preset("Default")
    ctrl.save_workspace("/test.json")
    ctrl.load_workspace("/test.json")

    ctrl.add_folder("/test")
    ctrl.remove_folder("/test")
    ctrl.edit_folder("/test", "/test2")
    ctrl.clear_folders()

    with patch.object(AsyncRuntime, 'run_coroutine'):
        ctrl.scan_only()
        ctrl.start_processing("clipboard")
        ctrl.copy_file_with_dependencies("/test2/file.py", "none")
        ctrl.add_github_repo("https://github", "")
        ctrl.verify_patch_with_llm({}, lambda x: None)

    ctrl.toggle_file_exclusion("/test2/file.py", True)
    ctrl.install_context_menu()
    ctrl.remove_context_menu()
    ctrl.close_preview()
    ctrl.prepare_llm_patch("[]")
    ctrl.apply_prepared_patches([])
    ctrl.parse_error_log("error in file.py")
    ctrl.show_tour()
    ctrl.close_tour()
    ctrl.copy_to_clipboard("test")
    ctrl.get_search_markers_for_preview("test.py")


def test_cli_controller():
    store = Store()
    dispatcher = Dispatcher(store)
    settings_repo = MagicMock()
    settings_repo.load.return_value = {}
    scan_uc = MagicMock()
    scan_uc.execute = AsyncMock()
    process_uc = MagicMock()
    process_uc.execute = AsyncMock()
    patch_uc = MagicMock()

    ctrl = CliController(store, dispatcher, settings_repo, scan_uc, process_uc, patch_uc)

    with patch("os.path.exists", return_value=True):
        with patch("asyncio.run"):
            ctrl.run("/path")
        with patch("builtins.open", MagicMock()):
            ctrl.run_patch("/path", "/patch.json")
