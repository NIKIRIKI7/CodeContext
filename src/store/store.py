"""
Store — центральное хранилище состояния (Redux-like).
"""
import copy
from typing import List, Callable, Any, Dict
from .state import AppState, AppSettings
from ..actions.action_types import *
from src.i18n import tr


class Store:
    """Центральное хранилище состояния."""
    def __init__(self):
        self._state = AppState()
        self._cached_state = None
        self._is_dirty = True
        self._listeners: List[Callable[[AppState], None]] = []
        self._handlers: Dict[str, Callable[[Any], None]] = {}
        self._register_handlers()

    @property
    def state(self) -> AppState:
        if self._is_dirty:
            self._cached_state = copy.deepcopy(self._state)
            self._is_dirty = False
        return self._cached_state

    def subscribe(self, listener: Callable[[AppState], None]) -> Callable:
        self._listeners.append(listener)
        return lambda: self._listeners.remove(listener)

    def dispatch(self, action_type: str, payload: Any = None):
        handler = self._handlers.get(action_type)
        if handler:
            self._is_dirty = True
            handler(payload)
            self._notify()

    def _register_handlers(self):
        self._handlers.update({
            UI_SET_LOADING:       lambda p: self._state.__setattr__('is_loading', bool(p)),
            UI_UPDATE_STATUS:     self._handle_update_status,
            UI_ADD_LOG:           lambda p: self._state.logs.append(str(p)),
            UI_SHOW_PREVIEW:      self._handle_show_preview,
            UI_CLOSE_PREVIEW:     self._handle_close_preview,
            UI_SHOW_TOUR:         self._handle_show_tour,
            UI_CLOSE_TOUR:        self._handle_close_tour,
            UI_SHOW_UPDATE:       self._handle_show_update,
            UI_CLOSE_UPDATE:      self._handle_close_update,
            UI_SHOW_TOAST:        self._handle_show_toast,
            UI_SHOW_CHAT:         self._handle_show_chat,
            UI_CLOSE_CHAT:        self._handle_close_chat,
            SETTINGS_LOADED:      self._handle_settings_loaded,
            SETTINGS_UPDATE:      self._handle_settings_update,
            WORKSPACE_LOADED:     self._handle_workspace_loaded,
            FOLDER_ADD:           self._handle_folder_add,
            FOLDER_REMOVE:        self._handle_folder_remove,
            FOLDER_UPDATE:        self._handle_folder_update,
            FOLDER_CLEAR:         self._handle_folder_clear,
            GITHUB_CLONE_SUCCESS: self._handle_github_success,
            GITHUB_CLONE_FAILURE: self._handle_github_failure,
            SCAN_SUCCESS:         self._handle_scan_success,
            SCAN_FAILURE:         self._handle_scan_failure,
            RECALCULATE_TOKENS:   self._handle_recalculate_tokens,
            EXCLUSION_ADD:        self._handle_exclusion_add,
            EXCLUSION_REMOVE:     self._handle_exclusion_remove,
            EXCLUSION_CLEAR:      self._handle_exclusion_clear,
            PROCESSING_SUCCESS:   lambda p: self._state.__setattr__('processed_files', p),
            FORMATTING_SUCCESS:   self._handle_formatting_success,
            WORKFLOW_STARTED:     self._handle_update_status,
            WORKFLOW_PROGRESS:    self._handle_update_status,
            WORKFLOW_FINISHED:    lambda _: self._handle_update_status({'message': tr("store.status.done"), 'progress': 1.0}),
            WORKFLOW_ERROR:       self._handle_workflow_error,
            HISTORY_ADD:          self._handle_history_add,
            SET_BEFORE_AFTER:     lambda p: self._state.__setattr__('before_after_data', p),
            UI_SHOW_COMMAND_PALETTE: lambda _: self._state.__setattr__('show_command_palette', True),
            UI_CLOSE_COMMAND_PALETTE: lambda _: self._state.__setattr__('show_command_palette', False),
            SET_PR_TARGET_FILES:  self._handle_set_pr_target_files,
        })

    def _handle_set_pr_target_files(self, payload: list):
        self._state.pr_target_files = payload

    def _handle_show_toast(self, message: str):
        self._state.toast_message = message

    def _handle_show_chat(self, payload: str):
        self._state.chat_context = payload
        self._state.show_chat = True

    def _handle_close_chat(self, _):
        self._state.show_chat = False
        self._state.chat_context = ""

    def _handle_update_status(self, payload: dict):
        if isinstance(payload, dict):
            self._state.status_message = payload.get('message', self._state.status_message)
            self._state.progress = payload.get('progress', self._state.progress)

    def _handle_show_preview(self, payload: str):
        self._state.preview_text = payload
        self._state.show_preview = True

    def _handle_close_preview(self, _):
        self._state.show_preview = False
        self._state.preview_text = ""

    def _handle_show_tour(self, steps: list):
        self._state.tour_steps = steps
        self._state.show_tour = True

    def _handle_close_tour(self, _):
        self._state.show_tour = False
        self._state.tour_steps = []

    def _handle_show_update(self, payload: dict):
        self._state.update_info = payload
        self._state.show_update = True

    def _handle_close_update(self, _):
        self._state.show_update = False
        self._state.update_info = {}

    def _handle_settings_loaded(self, payload: dict):
        valid_keys = set(AppSettings.__dataclass_fields__.keys())
        filtered = {k: v for k, v in payload.items() if k in valid_keys}

        if "approved_plugins" not in payload and "visible_tabs" in filtered:
            if "plugins" not in filtered["visible_tabs"]:
                filtered["visible_tabs"].append("plugins")

        self._state.settings = AppSettings(**filtered)
        lang = filtered.get('language', '')
        if not lang:
            from src.i18n import load_translations
            load_translations()
        else:
            from src.i18n import set_language
            set_language(lang)

    def _handle_settings_update(self, payload: dict):
        current = self._state.settings.__dict__.copy()
        valid_keys = set(AppSettings.__dataclass_fields__.keys())
        filtered = {k: v for k, v in payload.items() if k in valid_keys}
        current.update(filtered)
        self._state.settings = AppSettings(**current)

    def _handle_workspace_loaded(self, payload: dict):
        self._state.selected_folders = payload.get('folders', [])
        current = self._state.settings.__dict__.copy()
        settings_payload = payload.get('settings', {})
        valid_keys = set(AppSettings.__dataclass_fields__.keys())
        filtered = {k: v for k, v in settings_payload.items() if k in valid_keys}

        if "approved_plugins" not in settings_payload and "visible_tabs" in filtered:
            if "plugins" not in filtered["visible_tabs"]:
                filtered["visible_tabs"].append("plugins")

        current.update(filtered)
        self._state.settings = AppSettings(**current)
        self._state.temp_folders = []

    def _handle_folder_add(self, path: str):
        if path not in self._state.selected_folders:
            self._state.selected_folders.append(path)

    def _handle_folder_remove(self, path: str):
        if path in self._state.selected_folders:
            self._state.selected_folders.remove(path)
        if path in self._state.temp_folders:
            self._state.temp_folders.remove(path)

    def _handle_folder_update(self, payload: dict):
        old, new = payload['old'], payload['new']
        for col in (self._state.selected_folders, self._state.temp_folders):
            if old in col:
                col[col.index(old)] = new

    def _handle_folder_clear(self, _):
        self._state.selected_folders.clear()
        self._state.temp_folders.clear()
        self._state.scanned_files_paths.clear()
        self._state.scanned_file_metadata.clear()
        self._state.processed_files.clear()
        self._state.manual_exclusions.clear()
        self._state.pr_target_files.clear()
        self._state.final_output_text = ""
        self._state.total_tokens = 0
        self._state.selected_tokens = 0
        self._state.status_message = tr("store.status.workspace_cleared")
        self._state.progress = 0.0
        self._state.logs.clear()
        self._state.toast_message = ""

    def _handle_github_success(self, payload: dict):
        path = payload.get("path")
        is_temp = payload.get("is_temp", True)
        if path and path not in self._state.selected_folders:
            self._state.selected_folders.append(path)
        if path and is_temp and path not in self._state.temp_folders:
            self._state.temp_folders.append(path)
        self._state.status_message = tr("store.status.repo_loaded")
        self._state.logs.append(f"GitHub Repo Cloned: {path}")

    def _handle_github_failure(self, error: str):
        self._state.status_message = tr("store.status.clone_error")
        self._state.logs.append(f"GitHub Error: {error}")

    def _handle_scan_success(self, payload: dict):
        self._state.scanned_files_paths = payload['paths']
        self._state.scanned_file_metadata = payload['metadata']
        self._state.manual_exclusions = set()
        self._state.status_message = tr("store.status.files_found", count=len(payload['paths']))
        self._handle_recalculate_tokens(None)

    def _handle_scan_failure(self, error: str):
        self._state.scanned_files_paths.clear()
        self._state.status_message = tr("store.status.scan_error")
        self._state.logs.append(tr("store.status.scan_error_log", error=error))

    def _handle_formatting_success(self, payload: dict):
        self._state.final_output_text = payload['text']
        self._state.total_tokens = payload['tokens']
        self._state.status_message = tr("store.status.done")
        self._state.progress = 1.0
        self._state.is_loading = False

    def _handle_workflow_error(self, error: str):
        self._state.status_message = tr("store.status.error_with_msg", error_msg=error)
        self._state.is_loading = False
        self._state.logs.append(f"CRITICAL ERROR: {error}")

    def _handle_history_add(self, payload: dict):
        self._state.preview_history.insert(0, payload)
        if len(self._state.preview_history) > 20:
            self._state.preview_history.pop()

    def _handle_exclusion_add(self, path):
        self._state.manual_exclusions.add(path)
        self._state.final_output_text = ""
        self._handle_recalculate_tokens(None)

    def _handle_exclusion_remove(self, path):
        self._state.manual_exclusions.discard(path)
        self._state.final_output_text = ""
        self._handle_recalculate_tokens(None)

    def _handle_exclusion_clear(self, _):
        self._state.manual_exclusions.clear()
        self._state.final_output_text = ""
        self._handle_recalculate_tokens(None)

    def _handle_recalculate_tokens(self, _):
        total = 0
        for path, meta in self._state.scanned_file_metadata.items():
            if path not in self._state.manual_exclusions:
                total += meta.get("tokens", 0)
        self._state.selected_tokens = total

    def _notify(self):
        snapshot = self.state
        for listener in self._listeners:
            listener(snapshot)
