import os
import asyncio
from typing import Optional, Tuple, List
from PySide6.QtCore import QTimer
from src.i18n import tr
from ..store.state import AppState, ProcessedFile
from ..use_cases import scan_use_case, process_use_case, github_use_case, settings_use_case, patch_use_case, updater_use_case
from ..services import integration_service
from ..services.plugin_manager import PluginManager
from ..services import formatting_service
from ..services import output_service
from ..services.tour_service import TOUR_STEPS


class MainController:
    def __init__(
        self,
        state: AppState,
        plugin_manager: PluginManager,
        llm_checker,
    ):
        self.state = state
        self._llm_checker = llm_checker
        self._plugin_manager = plugin_manager

        self._plugin_tabs = {}
        self._plugin_actions = {}

    def register_sidebar_tab(self, tab_id: str, label: str, factory):
        self._plugin_tabs[tab_id] = {"id": tab_id, "label": label, "factory": factory}

    def register_action_button(self, action_id: str, label: str, callback):
        self._plugin_actions[action_id] = {"id": action_id, "label": label, "callback": callback}

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

        old_tabs = set(self._plugin_tabs.keys())
        old_actions = set(self._plugin_actions.keys())

        for manifest in manifests:
            if manifest.get("id") in approved:
                self._plugin_manager.load_plugin(manifest)

        new_tabs = set(self._plugin_tabs.keys()) - old_tabs
        new_actions = set(self._plugin_actions.keys()) - old_actions

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
        settings_use_case.load_initial(self.state)

    def update_settings(self, data: dict):
        settings_use_case.update_settings(self.state, data)

    def save_settings(self):
        settings_use_case.save(self.state)

    def reset_settings(self):
        settings_use_case.reset(self.state)

    def apply_preset(self, preset_name: str):
        settings_use_case.apply_preset(self.state, preset_name)

    def save_workspace(self, path: str):
        settings_use_case.save_workspace(self.state, path)

    def load_workspace(self, path: str):
        settings_use_case.load_workspace(self.state, path)
        self.scan_only()

    def add_folder(self, path: str):
        clean = self._normalize_path(path)
        if clean and os.path.exists(clean):
            if clean not in self.state.selected_folders:
                self.state.selected_folders.append(clean)
                self.state.notify()
            settings_use_case.load_local_config(self.state, clean)
            recent = list(self.state.settings.recent_workspaces)
            if clean in recent:
                recent.remove(clean)
            recent.insert(0, clean)
            recent = recent[:6]
            settings_use_case.update_settings(self.state, {'recent_workspaces': recent})
            settings_use_case.save(self.state)

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
        settings_use_case.load_initial(self.state)
        self.state.selected_folders.clear()
        self.state.notify()

    def scan_only(self):
        if not self.state.selected_folders:
            self.state.add_log(tr("main_controller.choose_folders"))
            return

        async def _scan_and_filter():
            await scan_use_case.scan_workspace(self.state)
            self._apply_pr_filter()

        asyncio.create_task(_scan_and_filter())

    def start_processing(self, target: str, save_path: Optional[str] = None) -> Tuple[bool, str]:
        if not self.state.selected_folders:
            return False, tr("main_controller.choose_folders_or_url")

        if not self.state.scanned_files_paths:
            asyncio.create_task(self._scan_then_process(target, save_path))
        else:
            asyncio.create_task(process_use_case.process_workspace(self.state, target, save_path))
        return True, ""

    async def _scan_then_process(self, target: str, save_path: Optional[str]):
        await scan_use_case.scan_workspace(self.state)
        self._apply_pr_filter()
        if self.state.scanned_files_paths:
            await process_use_case.process_workspace(self.state, target, save_path)

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
        asyncio.create_task(self._copy_deps_async(target_file, mode))

    async def _copy_deps_async(self, target_file: str, mode: str):
        if mode == 'none':
            from pathlib import Path
            try:
                content = Path(target_file).read_text(encoding='utf-8', errors='replace')
                pf = ProcessedFile(path=target_file, content=content, tokens=len(content)//4)
                text = formatting_service.format_output(
                    files=[pf],
                    fmt=self.state.settings.output_format,
                    include_tree=False,
                    system_prompt=""
                )
                self.copy_to_clipboard(text)
            except OSError:
                self.state.add_log(tr("main_controller.read_file_error", target_file=target_file))
        else:
            self.state.add_log(tr("main_controller.use_copy_deps_use_case", mode=mode))

    def add_github_repo(self, url: str, dest_path: str = ""):
        asyncio.create_task(github_use_case.github_use_case(self.state, url, dest_path))

    def save_local_config(self):
        if self.state.selected_folders:
            settings_use_case.save_local_config(self.state, self.state.selected_folders[0])
        else:
            self.state.add_log(tr("main_controller.no_folders_config"))

    def install_context_menu(self) -> Tuple[bool, str]:
        return integration_service.install_context_menu(self.state.settings.python_interpreter)

    def remove_context_menu(self) -> Tuple[bool, str]:
        return integration_service.remove_context_menu()

    def install_cli(self) -> Tuple[bool, str]:
        return integration_service.install_cli(self.state.settings.python_interpreter)

    def remove_cli(self) -> Tuple[bool, str]:
        return integration_service.remove_cli()

    def close_preview(self):
        self.state.show_preview = False
        self.state.notify()

    def close_chat(self):
        self.state.show_chat = False
        self.state.notify()

    def prepare_llm_patch(self, json_str: str) -> list:
        return patch_use_case.prepare_json_patch(self.state, json_str, self.state.selected_folders)

    def verify_patch_with_llm(self, patch: dict, callback):
        async def task():
            verdict = await self._llm_checker.check_patch(
                patch.get('original_content', ''),
                patch.get('patched_content', ''),
                self.state.settings
            )
            QTimer.singleShot(0, lambda: callback(verdict))
        asyncio.create_task(task())

    def apply_prepared_patches(self, prepared_patches: list):
        patch_use_case.apply_patches(self.state, prepared_patches)

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
        self.state.tour_steps = TOUR_STEPS
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
        asyncio.create_task(updater_use_case.check_for_updates(self.state, current_version))

    def apply_update(self, download_url: str):
        asyncio.create_task(updater_use_case.apply_update(self.state, download_url))

    def close_update_dialog(self):
        self.state.show_update = False
        self.state.notify()

    @staticmethod
    def _normalize_path(path: str) -> str:
        if not path: return ""
        return os.path.abspath(path.strip('"\''))

    def add_log(self, text: str):
        self.state.add_log(text)
