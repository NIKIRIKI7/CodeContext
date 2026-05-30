import pytest
from src.store.store import Store
from src.actions.dispatcher import Dispatcher
from src.actions.action_types import *


def test_store_and_dispatcher():
    store = Store()
    dispatcher = Dispatcher(store)

    dispatcher.dispatch(UI_SET_LOADING, True)
    assert store.state.is_loading is True

    dispatcher.dispatch(UI_UPDATE_STATUS, {'message': 'Test', 'progress': 0.5})
    assert store.state.status_message == 'Test'

    dispatcher.dispatch(UI_ADD_LOG, 'Log')
    assert 'Log' in store.state.logs

    dispatcher.dispatch(UI_SHOW_PREVIEW, "Prev")
    assert store.state.show_preview is True

    dispatcher.dispatch(UI_CLOSE_PREVIEW, None)
    assert store.state.show_preview is False

    dispatcher.dispatch(UI_SHOW_TOUR, [])
    assert store.state.show_tour is True

    dispatcher.dispatch(UI_CLOSE_TOUR, None)
    assert store.state.show_tour is False

    dispatcher.dispatch(SETTINGS_LOADED, {'minify': False})
    assert store.state.settings.minify is False

    dispatcher.dispatch(SETTINGS_UPDATE, {'minify': True})
    assert store.state.settings.minify is True

    dispatcher.dispatch(WORKSPACE_LOADED, {'folders': ['/a'], 'settings': {'minify': False}})
    assert '/a' in store.state.selected_folders
    assert store.state.settings.minify is False

    dispatcher.dispatch(FOLDER_ADD, '/b')
    assert '/b' in store.state.selected_folders

    dispatcher.dispatch(FOLDER_REMOVE, '/a')
    assert '/a' not in store.state.selected_folders

    dispatcher.dispatch(FOLDER_UPDATE, {'old': '/b', 'new': '/c'})
    assert '/c' in store.state.selected_folders

    dispatcher.dispatch(FOLDER_CLEAR, None)
    assert len(store.state.selected_folders) == 0

    dispatcher.dispatch(GITHUB_CLONE_SUCCESS, {'path': '/repo', 'is_temp': True})
    assert '/repo' in store.state.selected_folders
    dispatcher.dispatch(GITHUB_CLONE_FAILURE, "Err")

    dispatcher.dispatch(SCAN_SUCCESS, {'paths': ['/p'], 'metadata': {'/p': {}}})
    assert '/p' in store.state.scanned_files_paths

    dispatcher.dispatch(SCAN_FAILURE, "Err")
    assert len(store.state.scanned_files_paths) == 0

    dispatcher.dispatch(EXCLUSION_ADD, '/p')
    assert '/p' in store.state.manual_exclusions

    dispatcher.dispatch(EXCLUSION_REMOVE, '/p')
    assert '/p' not in store.state.manual_exclusions
    dispatcher.dispatch(EXCLUSION_CLEAR, None)

    dispatcher.dispatch(PROCESSING_SUCCESS, [])
    assert store.state.processed_files == []

    dispatcher.dispatch(FORMATTING_SUCCESS, {'text': 'T', 'tokens': 1})
    assert store.state.final_output_text == 'T'

    dispatcher.dispatch(WORKFLOW_STARTED, {'message': 'start', 'progress': 0})
    dispatcher.dispatch(WORKFLOW_PROGRESS, {'message': 'mid', 'progress': 0.5})
    dispatcher.dispatch(WORKFLOW_FINISHED, None)
    assert store.state.progress == 1.0

    dispatcher.dispatch(WORKFLOW_ERROR, "Err")
    dispatcher.dispatch(HISTORY_ADD, {"time": "1", "text": "2", "tokens": 3})
    dispatcher.dispatch(SET_BEFORE_AFTER, [])

    store._state.settings.enable_logging = True
    dispatcher.dispatch(UI_ADD_LOG, "A" * 400)

    store._state.settings.enable_logging = False
    dispatcher.dispatch(UI_ADD_LOG, "Hidden Log")
    assert "Hidden Log" in store.state.logs
