import os
from typing import Optional, Tuple, List
from PySide6.QtCore import QTimer
from src.i18n import tr

from ..store.state import AppState, ProcessedFile
from ..use_cases.scan_use_case import ScanWorkspaceUseCase
from ..use_cases.process_use_case import ProcessWorkspaceUseCase
from ..use_cases.github_use_case import GitHubUseCase
from ..use_cases.settings_use_case import SettingsUseCase
from ..use_cases.patch_use_case import PatchUseCase
from ..use_cases.updater_use_case import UpdaterUseCase
from ..utils.async_runtime import AsyncRuntime
from ..data.file_system_repository import FileSystemRepository
from ..api.plugin_api import PluginAPI
from ..services.plugin_manager import PluginManager

from ..services import formatting_service
from ..services import output_service

class MainController:
    def __init__(
        self,
        state: AppState,
        scan_use_case: ScanWorkspaceUseCase,
        process_use_case: ProcessWorkspaceUseCase,
        github_use_case: GitHubUseCase,
        settings_use_case: SettingsUseCase,
        patch_use_case: PatchUseCase,
        updater_use_case: UpdaterUseCase,
        integration_strategy,
        fs_repo: FileSystemRepository,
        tour_service,
        llm_checker,
        plugin_api: PluginAPI,
        plugin_manager: PluginManager
    ):
        self.state = state
        self._scan_uc = scan_use_case
        self._process_uc = process_use_case
        self._github_uc = github_use_case
        self._settings_uc = settings_use_case
        self._patch_uc = patch_use_case
        self._updater_uc = updater_use_case
        self._integration_strategy = integration_strategy
        self._fs_repo = fs_repo
        self._tour_service = tour_service
        self._llm_checker = llm_checker
        self._plugin_api = plugin_api
        self._plugin_manager = plugin_manager

    def init_plugins(self, parent_widget):
        manifests = self._plugin_manager.discover_plugins()
        settings = self.state.settings
        approved = list(settings.approved_plugins)
        changed = False

        for manifest in manifests:
            p_id = manifest.get("id")
            
            if p_id not in approved:
                from src.ui.dialogs import PluginApprovalDialog
                dlg = PluginApprovalDialog(parent_widget, manifest)
                if dlg.exec():
                    approved.append(p_id)
                    changed = True
                    
                    req_path = os.path.join(manifest['_dir'], "requirements.txt")
                    if os.path.exists(req_path):
                        from src.ui.dialogs import PluginInstallDialog
                        install_dlg = PluginInstallDialog(parent_widget, manifest, req_path)
                        install_dlg.exec()
                else:
                    continue

            old_tabs = set(self._plugin_api.ui.sidebar_tabs.keys())
            old_actions = set(self._plugin_api.ui.action_buttons.keys())

            self._plugin_manager.load_plugin(manifest)

            new_tabs = set(self._plugin_api.ui.sidebar_tabs.keys()) - old_tabs
            new_actions = set(self._plugin_api.ui.action_buttons.keys()) - old_actions

            if new_tabs or new_actions:
                vis_tabs = list(self.state.settings.visible_tabs)
                vis_acts = list(self.state.settings.visible_actions)
                needs_save = False

                for t in new_tabs:
                    if t not in vis_tabs:
                        vis_tabs.append(t)
                        needs_save = True
                for a in new_actions:
                    if a not in vis_acts:
                        vis_acts.append(a)
                        needs_save = True

                if needs_save:
                    self.update_settings({"visible_tabs": vis_tabs, "visible_actions": vis_acts})
                    changed = True

        if changed:
            self.update_settings({"approved_plugins": approved})
            self.save_settings()

    def shutdown(self):
        self._plugin_manager.shutdown_all()

    def load_initial_settings(self):
        self._settings_uc.load_initial()

    def update_settings(self, data: dict):
        self._settings_uc.update(data)

    def save_settings(self):
        self._settings_uc.save()

    def reset_settings(self):
        self._settings_uc.reset()

    def apply_preset(self, preset_name: str):
        self._settings_uc.apply_preset(preset_name)

    def save_workspace(self, path: str):
        self._settings_uc.save_workspace(path)

    def load_workspace(self, path: str):
        self._settings_uc.load_workspace(path)
        self.scan_only()

    def add_folder(self, path: str):
        clean = self._normalize_path(path)
        if clean and os.path.exists(clean):
            if clean not in self.state.selected_folders:
                self.state.selected_folders.append(clean)
                self.state.notify()
            self._settings_uc.load_local_config(clean)
            recent = list(self.state.settings.recent_workspaces)
            if clean in recent:
                recent.remove(clean)
            recent.insert(0, clean)
            recent = recent[:6]
            self._settings_uc.update({'recent_workspaces': recent})
            self._settings_uc.save()

    def remove_folder(self, path: str):
        if path in self.state.selected_folders:
            self.state.selected_folders.remove(path)
            self.state.notify()

    def edit_folder(self, old_path: str, new_path: str):
        clean = self._normalize_path(new_path)
        if clean and clean != old_path:
            if old_path in self.state.selected_folders:
                idx = self.state.selected_folders.index(old_path)
                self.state.selected_folders[idx] = clean
                self.state.notify()

    def clear_folders(self):
        self._settings_uc.load_initial()
        temp = self.state.temp_folders
        if temp:
            self.state.add_log(tr("main_controller.clear_temp_files"))
            AsyncRuntime.run_coroutine(self._clear_temp_async(temp))
        else:
            self.state.selected_folders.clear()
            self.state.notify()

    async def _clear_temp_async(self, folders):
        for folder in folders:
            await self._fs_repo.delete_directory_async(folder)
        self.state.selected_folders.clear()
        self.state.notify()

    def scan_only(self):
        if not self.state.selected_folders:
            self.state.add_log(tr("main_controller.choose_folders"))
            return
        async def _scan_and_filter():
            await self._scan_uc.execute(self.state)
            self._apply_pr_filter()
        AsyncRuntime.run_coroutine(_scan_and_filter())

    def start_processing(self, target: str, save_path: Optional[str] = None) -> Tuple[bool, str]:
        if not self.state.selected_folders:
            return False, tr("main_controller.choose_folders_or_url")

        if not self.state.scanned_files_paths:
            AsyncRuntime.run_coroutine(self._scan_then_process(target, save_path))
        else:
            AsyncRuntime.run_coroutine(self._process_uc.execute(self.state, target, save_path))
        return True, ""

    async def _scan_then_process(self, target: str, save_path: Optional[str]):
        await self._scan_uc.execute(self.state)
        self._apply_pr_filter()
        if self.state.scanned_files_paths:
            await self._process_uc.execute(self.state, target, save_path)

    def toggle_file_exclusion(self, path: str, is_included: bool):
        if is_included:
            self.state.manual_exclusions.discard(path)
        else:
            self.state.manual_exclusions.add(path)
        self.state.final_output_text = ""

        total = sum(meta.get("tokens", 0) for p, meta in self.state.scanned_file_metadata.items() if p not in self.state.manual_exclusions)
        self.state.selected_tokens = total
        self.state.notify()

    def copy_file_with_dependencies(self, target_file: str, mode: str):
        AsyncRuntime.run_coroutine(self._copy_deps_async(target_file, mode))

    async def _copy_deps_async(self, target_file: str, mode: str):
        if mode == 'none':
            content = await self._fs_repo.read_file_async(target_file)
            if content is not None:
                pf = ProcessedFile(path=target_file, content=content, tokens=len(content)//4)
                text = formatting_service.format_output(
                    files=[pf],
                    fmt=self.state.settings.output_format,
                    include_tree=False,
                    system_prompt=""
                )
                self.copy_to_clipboard(text)
            else:
                self.state.add_log(tr("main_controller.read_file_error", target_file=target_file))
        else:
            self.state.add_log(tr("main_controller.use_copy_deps_use_case", mode=mode))

    def add_github_repo(self, url: str, dest_path: str = ""):
        AsyncRuntime.run_coroutine(self._github_uc.execute(url, dest_path))

    def save_local_config(self):
        if self.state.selected_folders:
            self._settings_uc.save_local_config(self.state.selected_folders[0])
        else:
            self.state.add_log(tr("main_controller.no_folders_config"))

    def install_context_menu(self) -> Tuple[bool, str]:
        python_path = self.state.settings.python_interpreter
        return self._integration_strategy.install(python_path)

    def remove_context_menu(self) -> Tuple[bool, str]:
        return self._integration_strategy.remove()

    def install_cli(self) -> Tuple[bool, str]:
        python_path = self.state.settings.python_interpreter
        return self._integration_strategy.install_cli(python_path)

    def remove_cli(self) -> Tuple[bool, str]:
        return self._integration_strategy.remove_cli()

    def close_preview(self):
        self.state.show_preview = False
        self.state.notify()

    def close_chat(self):
        self.state.show_chat = False
        self.state.notify()

    def prepare_llm_patch(self, json_str: str) -> list:
        return self._patch_uc.prepare_json_patch(json_str, self.state.selected_folders)

    def verify_patch_with_llm(self, patch: dict, callback):
        async def task():
            verdict = await self._llm_checker.check_patch(
                patch.get('original_content', ''),
                patch.get('patched_content', ''),
                self.state.settings
            )
            QTimer.singleShot(0, lambda: callback(verdict))
        AsyncRuntime.run_coroutine(task())

    def apply_prepared_patches(self, prepared_patches: list):
        self._patch_uc.apply_prepared(prepared_patches)

    def parse_error_log(self, text: str):
        if not self.state.scanned_files_paths:
            self.state.add_log(tr("main_controller.scan_first"))
            return
        matched = []
        for path in self.state.scanned_files_paths:
            basename = os.path.basename(path)
            if basename in text:
                matched.append(path)
        if not matched:
            self.state.add_log(tr("main_controller.no_files_in_log"))
            return

        self.state.manual_exclusions.clear()
        all_paths = set(self.state.scanned_files_paths)
        for p in (all_paths - set(matched)):
            self.state.manual_exclusions.add(p)

        self.state.add_log(tr("main_controller.files_matched", count=len(matched)))
        self.state.notify()

    def show_tour(self):
        self.state.tour_steps = self._tour_service.get_tour_steps()
        self.state.show_tour = True
        self.state.notify()

    def close_tour(self):
        self.state.show_tour = False
        self.state.notify()

    def copy_to_clipboard(self, text: str):
        try:
            output_service.copy_to_clipboard(text)
            tokens = self.state.selected_tokens if self.state.selected_tokens > 0 else self.state.total_tokens
            self.state.add_log(tr("main_controller.copied_to_clipboard"))
            self.state.toast_message = tr("main_controller.copied_tokens", tokens=tokens)
            self.state.notify()
        except Exception as e:
            self.state.add_log(tr("main_controller.clipboard_error", error=str(e)))
            self.state.toast_message = tr("main_controller.clipboard_error_generic")
            self.state.notify()

    def _apply_pr_filter(self):
        if not self.state.pr_target_files or not self.state.scanned_files_paths:
            return

        self.state.manual_exclusions.clear()
        pr_files_norm = [p.replace('/', os.sep) for p in self.state.pr_target_files]
        for p in self.state.scanned_files_paths:
            if not any(p.endswith(pr_f) for pr_f in pr_files_norm):
                self.state.manual_exclusions.add(p)

        self.state.add_log(tr("main_controller.pr_filter_applied", count=len(self.state.pr_target_files)))
        self.state.pr_target_files.clear()
        self.state.notify()

    def clear_toast(self):
        self.state.toast_message = ""

    def get_search_markers_for_preview(self, filepath: str) -> List[str]:
        return formatting_service.get_search_markers(filepath)

    def generate_html_diff(self, source_text: str, target_text: str, colors: dict, fonts: dict) -> str:
        return formatting_service.generate_html_diff(source_text, target_text, colors, fonts)

    def check_for_updates(self, current_version: str):
        AsyncRuntime.run_coroutine(self._updater_uc.check_for_updates(self.state, current_version))

    def apply_update(self, download_url: str):
        AsyncRuntime.run_coroutine(self._updater_uc.apply_update(download_url))

    def close_update_dialog(self):
        self.state.show_update = False
        self.state.notify()

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path: return ""
        clean = path
        while True:
            prev = clean
            clean = clean.strip().strip('"\'')
            if clean == prev:
                break
        try:
            return os.path.normpath(clean)
        except (TypeError, ValueError, OSError):
            return clean
