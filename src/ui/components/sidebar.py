import os
import sys
import shutil
import platform
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QTabWidget, QFormLayout,
                               QCheckBox, QComboBox, QLineEdit, QTextEdit,
                               QPushButton, QHBoxLayout, QLabel, QFileDialog,
                               QInputDialog, QMessageBox, QPlainTextEdit)
from PySide6.QtCore import Qt

from ...utils.config import PRESETS, PROMPT_PRESETS, get_app_version
from ..theme_manager import ThemeManager, theme_bus
from ..dialogs import UICustomizationDialog
from src.i18n import tr, set_language, current_lang, available_languages


class Sidebar(QWidget):
    TAB_DEFS = [
        ("sources", "sidebar.tab.sources", "_build_sources_tab"),
        ("filters", "sidebar.tab.filters", "_build_filters_tab"),
        ("prompts", "sidebar.tab.prompts", "_build_prompts_tab"),
        ("llm_os", "sidebar.tab.llm_os", "_build_llm_os_tab"),
        ("appearance", "sidebar.tab.appearance", "_build_appearance_tab"),
    ]

    def __init__(self, controller, on_settings_change):
        super().__init__()
        self.controller = controller
        self.on_settings_change = on_settings_change

        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setProperty("cssClass", "card")

        self.layout = QVBoxLayout(self)

        title = QLabel("CodeContext AI")
        title.setProperty("cssClass", "heading")
        self.layout.addWidget(title)

        self.tabs = QTabWidget()
        self.layout.addWidget(self.tabs)

        visible_tabs = getattr(self.controller._store.state.settings, 'visible_tabs', ["sources", "filters", "prompts", "llm_os", "appearance"])
        self._rebuild_tabs(visible_tabs)

        bottom_layout = QHBoxLayout()
        btn_ui_settings = QPushButton("\u2699")
        btn_ui_settings.setProperty("cssClass", "icon")
        btn_ui_settings.setToolTip(tr("sidebar.ui_settings.tooltip"))
        btn_ui_settings.clicked.connect(self._open_ui_settings)

        btn_tour = QPushButton(tr("sidebar.tour.button"))
        btn_tour.setProperty("cssClass", "ghost")
        btn_tour.clicked.connect(self.controller.show_tour)

        version_str = get_app_version()
        lbl_version = QLabel(tr("sidebar.version.label", version=version_str))
        lbl_version.setProperty("cssClass", "muted")

        btn_update = QPushButton(tr("sidebar.update.button"))
        btn_update.setProperty("cssClass", "ghost")
        btn_update.clicked.connect(self._check_updates)

        bottom_layout.addWidget(btn_ui_settings)
        bottom_layout.addWidget(lbl_version)
        bottom_layout.addStretch()
        bottom_layout.addWidget(btn_update)
        bottom_layout.addWidget(btn_tour)
        self.layout.addLayout(bottom_layout)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _open_ui_settings(self):
        settings = self.controller._store.state.settings
        dialog = UICustomizationDialog(self, settings, self._on_ui_settings_saved)
        dialog.exec()

    def _on_ui_settings_saved(self, visible_tabs, visible_actions):
        self.controller.update_settings({
            'visible_tabs': visible_tabs,
            'visible_actions': visible_actions,
        })
        self.controller.save_settings()

    def _rebuild_tabs(self, visible_tabs):
        self.tabs.blockSignals(True)
        self.tabs.clear()

        self.tab_widgets = {}
        for tab_id, label, method_name in self.TAB_DEFS:
            tab = QWidget()
            getattr(self, method_name)(tab)
            self.tab_widgets[tab_id] = tab
            if tab_id in visible_tabs:
                self.tabs.addTab(tab, tr(label))

        self._current_visible_tabs = visible_tabs
        self.tabs.blockSignals(False)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setContentsMargins(m, m, m, m)
        self.layout.setSpacing(s)

    def _build_sources_tab(self, tab):
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 10, 0, 0)

        btn_layout = QHBoxLayout()
        btn_layout.setContentsMargins(0, 0, 0, 10)

        btn_add = QPushButton(tr("sidebar.sources.add_folder"))
        btn_add.setProperty("cssClass", "ghost")
        btn_add.clicked.connect(self._add_folder)

        btn_gh = QPushButton(tr("sidebar.sources.add_github"))
        btn_gh.setProperty("cssClass", "success")
        btn_gh.setToolTip(tr("sidebar.sources.github_tooltip"))
        btn_gh.clicked.connect(self._add_github)

        btn_layout.addWidget(btn_add)
        btn_layout.addWidget(btn_gh)
        layout.addLayout(btn_layout)

        self.chk_git = QCheckBox(tr("sidebar.sources.git_only"))
        self.chk_gitignore = QCheckBox(tr("sidebar.sources.respect_gitignore"))
        layout.addWidget(self.chk_git)
        layout.addWidget(self.chk_gitignore)

        layout.addSpacing(10)
        btn_scan = QPushButton(tr("sidebar.sources.scan_files"))
        btn_scan.clicked.connect(self._trigger_scan)
        layout.addWidget(btn_scan)

        btn_save_local = QPushButton(tr("sidebar.sources.save_config"))
        btn_save_local.setProperty("cssClass", "ghost")
        btn_save_local.clicked.connect(self.controller.save_local_config)
        layout.addWidget(btn_save_local)

        btn_clear = QPushButton(tr("sidebar.sources.clear_project"))
        btn_clear.setProperty("cssClass", "ghost")
        btn_clear.clicked.connect(self.controller.clear_folders)
        layout.addWidget(btn_clear)

        layout.addStretch()

    def _build_filters_tab(self, tab):
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 10, 0, 0)

        preset_layout = QHBoxLayout()
        self.cmb_preset = QComboBox()
        self.cmb_preset.setSizeAdjustPolicy(QComboBox.AdjustToContents)
        self.cmb_preset.currentTextChanged.connect(self._on_ext_preset_change)

        btn_save_preset = QPushButton("\U0001F4BE")
        btn_save_preset.setProperty("cssClass", "icon")
        btn_save_preset.setToolTip(tr("sidebar.filters.save_preset_tooltip"))
        btn_save_preset.clicked.connect(self._save_ext_preset)

        btn_del_preset = QPushButton("\U0001F5D1")
        btn_del_preset.setProperty("cssClass", "icon")
        btn_del_preset.setToolTip(tr("sidebar.filters.delete_preset_tooltip"))
        btn_del_preset.clicked.connect(self._del_ext_preset)

        preset_layout.addWidget(QLabel(tr("sidebar.filters.preset_label")))
        preset_layout.addWidget(self.cmb_preset, 1)
        preset_layout.addWidget(btn_save_preset)
        preset_layout.addWidget(btn_del_preset)
        layout.addLayout(preset_layout)

        layout.addWidget(QLabel(tr("sidebar.filters.extensions_label")))
        self.entry_ext = QPlainTextEdit()
        self.entry_ext.setProperty("cssClass", "textarea_small")
        self.entry_ext.setPlaceholderText(tr("sidebar.filters.extensions_placeholder"))
        layout.addWidget(self.entry_ext)

        layout.addWidget(QLabel(tr("sidebar.filters.ignore_paths_label")))
        self.entry_ign = QPlainTextEdit()
        self.entry_ign.setProperty("cssClass", "textarea_small")
        self.entry_ign.setPlaceholderText(tr("sidebar.filters.ignore_paths_placeholder"))
        layout.addWidget(self.entry_ign)

        self.chk_tree = QCheckBox(tr("sidebar.filters.include_tree"))
        self.chk_dependencies = QCheckBox(tr("sidebar.filters.include_deps"))
        self.chk_mermaid = QCheckBox(tr("sidebar.filters.include_mermaid"))
        layout.addWidget(self.chk_tree)
        layout.addWidget(self.chk_dependencies)
        layout.addWidget(self.chk_mermaid)

        layout.addStretch()

    def _build_prompts_tab(self, tab):
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 10, 0, 0)

        preset_layout = QHBoxLayout()
        self.cmb_prompt = QComboBox()
        self.cmb_prompt.currentTextChanged.connect(self._on_prompt_preset_change)

        btn_save_prompt = QPushButton("\U0001F4BE")
        btn_save_prompt.setProperty("cssClass", "icon")
        btn_save_prompt.setToolTip(tr("sidebar.prompts.save_preset_tooltip"))
        btn_save_prompt.clicked.connect(self._save_prompt_preset)

        btn_del_prompt = QPushButton("\U0001F5D1")
        btn_del_prompt.setProperty("cssClass", "icon")
        btn_del_prompt.setToolTip(tr("sidebar.prompts.delete_preset_tooltip"))
        btn_del_prompt.clicked.connect(self._del_prompt_preset)

        preset_layout.addWidget(QLabel(tr("sidebar.prompts.preset_label")))
        preset_layout.addWidget(self.cmb_prompt, 1)
        preset_layout.addWidget(btn_save_prompt)
        preset_layout.addWidget(btn_del_prompt)
        layout.addLayout(preset_layout)

        layout.addWidget(QLabel(tr("sidebar.prompts.system_prompt_label")))
        self.txt_system_prompt = QTextEdit()
        layout.addWidget(self.txt_system_prompt)

        btn_patch = QPushButton(tr("sidebar.prompts.apply_patch"))
        btn_patch.setProperty("cssClass", "success")
        btn_patch.clicked.connect(self._open_patch_dialog)
        layout.addWidget(btn_patch)

    def _build_llm_os_tab(self, tab):
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 10, 0, 0)

        lbl_llm = QLabel(tr("sidebar.llm_os.llm_validator"))
        lbl_llm.setProperty("cssClass", "heading")
        layout.addWidget(lbl_llm)

        self.chk_llm_check = QCheckBox(tr("sidebar.llm_os.enable_llm_check"))
        layout.addWidget(self.chk_llm_check)

        form_llm = QFormLayout()
        form_llm.setContentsMargins(0, 0, 0, 0)
        self.entry_llm_url = QLineEdit()
        self.entry_llm_url.setPlaceholderText("https://api.openai.com/v1")
        self.entry_llm_key = QLineEdit()
        self.entry_llm_key.setEchoMode(QLineEdit.Password)
        self.entry_llm_key.setPlaceholderText("sk-...")
        self.entry_llm_model = QLineEdit()
        self.entry_llm_model.setPlaceholderText("gpt-4o-mini / local-model")
        form_llm.addRow(tr("sidebar.llm_os.url_label"), self.entry_llm_url)
        form_llm.addRow(tr("sidebar.llm_os.key_label"), self.entry_llm_key)
        form_llm.addRow(tr("sidebar.llm_os.model_label"), self.entry_llm_model)
        layout.addLayout(form_llm)

        llm_presets_layout = QHBoxLayout()
        btn_ollama = QPushButton(tr("sidebar.llm_os.ollama_preset"))
        btn_ollama.setProperty("cssClass", "ghost")
        btn_ollama.clicked.connect(lambda: self._apply_llm_preset("http://localhost:11434/v1", "llama3"))

        btn_lmstudio = QPushButton(tr("sidebar.llm_os.lmstudio_preset"))
        btn_lmstudio.setProperty("cssClass", "ghost")
        btn_lmstudio.clicked.connect(lambda: self._apply_llm_preset("http://localhost:1234/v1", "local-model"))

        llm_presets_layout.addWidget(btn_ollama)
        llm_presets_layout.addWidget(btn_lmstudio)
        llm_presets_layout.addStretch()
        layout.addLayout(llm_presets_layout)

        layout.addSpacing(10)
        lbl_os = QLabel(tr("sidebar.llm_os.os_integration", os_name=platform.system()))
        lbl_os.setProperty("cssClass", "heading")
        layout.addWidget(lbl_os)

        btn_ctx_layout = QHBoxLayout()
        self.btn_install_ctx = QPushButton(tr("sidebar.llm_os.install_context_menu"))
        self.btn_install_ctx.setProperty("cssClass", "success")
        self.btn_install_ctx.clicked.connect(self._install_context_menu)
        self.btn_remove_ctx = QPushButton(tr("sidebar.llm_os.remove_context_menu"))
        self.btn_remove_ctx.setProperty("cssClass", "ghost")
        self.btn_remove_ctx.clicked.connect(self._remove_context_menu)
        btn_ctx_layout.addWidget(self.btn_install_ctx)
        btn_ctx_layout.addWidget(self.btn_remove_ctx)
        layout.addLayout(btn_ctx_layout)

        layout.addSpacing(10)
        lbl_cli = QLabel(tr("sidebar.llm_os.cli_global"))
        lbl_cli.setProperty("cssClass", "heading")
        layout.addWidget(lbl_cli)

        btn_cli_layout = QHBoxLayout()
        self.btn_install_cli = QPushButton(tr("sidebar.llm_os.install_cli"))
        self.btn_install_cli.setProperty("cssClass", "success")
        self.btn_install_cli.clicked.connect(self._install_cli)
        self.btn_remove_cli = QPushButton(tr("sidebar.llm_os.remove_cli"))
        self.btn_remove_cli.setProperty("cssClass", "ghost")
        self.btn_remove_cli.clicked.connect(self._remove_cli)
        btn_cli_layout.addWidget(self.btn_install_cli)
        btn_cli_layout.addWidget(self.btn_remove_cli)
        layout.addLayout(btn_cli_layout)

        layout.addSpacing(10)
        editor_form = QFormLayout()
        self.entry_editor = QLineEdit()
        self.entry_editor.setPlaceholderText(tr("sidebar.llm_os.editor_placeholder"))
        editor_form.addRow(tr("sidebar.llm_os.editor_label"), self.entry_editor)
        layout.addLayout(editor_form)

        layout.addSpacing(10)
        lbl_upd = QLabel(tr("sidebar.llm_os.update_settings"))
        lbl_upd.setProperty("cssClass", "heading")
        layout.addWidget(lbl_upd)

        self.chk_prerelease = QCheckBox(tr("sidebar.llm_os.prerelease"))
        layout.addWidget(self.chk_prerelease)

        layout.addStretch()

    def _build_appearance_tab(self, tab):
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 10, 0, 0)

        form = QFormLayout()
        form.setContentsMargins(0, 0, 0, 0)

        self.cmb_theme = QComboBox()
        self.cmb_theme.currentTextChanged.connect(lambda t: ThemeManager.apply_theme(theme_id=t))

        self.cmb_mode = QComboBox()
        self.cmb_mode.addItems(["light", "dark"])
        self.cmb_mode.setCurrentText(ThemeManager._current_mode)
        self.cmb_mode.currentTextChanged.connect(lambda m: ThemeManager.apply_theme(mode=m))

        self.cmb_lang = QComboBox()
        self._lang_codes = list(available_languages().keys())
        self.cmb_lang.addItems([f"{code} - {available_languages()[code]}" for code in self._lang_codes])

        current_code = current_lang()
        idx = next((i for i, c in enumerate(self._lang_codes) if c == current_code), 0)
        self.cmb_lang.setCurrentIndex(idx)
        self.cmb_lang.currentIndexChanged.connect(self._on_language_change)

        form.addRow(tr("sidebar.appearance.theme_label"), self.cmb_theme)
        form.addRow(tr("sidebar.appearance.mode_label"), self.cmb_mode)
        form.addRow(tr("sidebar.appearance.language_label"), self.cmb_lang)
        layout.addLayout(form)

        layout.addSpacing(20)
        btn_themes_folder = QPushButton(tr("sidebar.appearance.open_themes_folder"))
        btn_themes_folder.setProperty("cssClass", "ghost")
        btn_themes_folder.clicked.connect(self._open_themes_folder)

        btn_import_theme = QPushButton(tr("sidebar.appearance.import_theme"))
        btn_import_theme.setProperty("cssClass", "success")
        btn_import_theme.clicked.connect(self._import_theme)

        layout.addWidget(btn_themes_folder)
        layout.addWidget(btn_import_theme)
        layout.addStretch()

    def _on_language_change(self, idx):
        if idx < 0 or idx >= len(self._lang_codes):
            return
        new_lang = self._lang_codes[idx]
        if new_lang == current_lang():
            return

        set_language(new_lang)
        self._current_visible_tabs = None
        self.controller.update_settings({'language': new_lang})
        self.controller.save_settings()

        mw = self.window()
        if hasattr(mw, 'retranslate_ui'):
            mw.retranslate_ui()

    def _add_folder(self):
        path = QFileDialog.getExistingDirectory(self, tr("sidebar.add_folder.title"))
        if path:
            self.controller.add_folder(path)

    def _apply_llm_preset(self, url: str, model: str):
        self.entry_llm_url.setText(url)
        self.entry_llm_key.setText("not-needed")
        self.entry_llm_model.setText(model)
        self.chk_llm_check.setChecked(True)
        self.on_settings_change()

    def _add_github(self):
        url, ok = QInputDialog.getText(self, "GitHub", tr("sidebar.add_github.prompt"))
        if ok and url:
            reply = QMessageBox.question(
                self, tr("sidebar.add_github.save_title"),
                tr("sidebar.add_github.save_prompt"),
                QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel
            )
            if reply == QMessageBox.Yes:
                dest_dir = QFileDialog.getExistingDirectory(self, tr("sidebar.add_github.clone_dialog"))
                if dest_dir:
                    self.controller.add_github_repo(url, dest_dir)
            elif reply == QMessageBox.No:
                self.controller.add_github_repo(url, "")

    def _trigger_scan(self):
        self.on_settings_change()
        self.controller.scan_only()

    def _install_context_menu(self):
        self.on_settings_change()
        success, msg = self.controller.install_context_menu()
        if success:
            QMessageBox.information(self, tr("sidebar.integration.title"), msg)
        else:
            QMessageBox.warning(self, tr("sidebar.error.title"), msg)

    def _remove_context_menu(self):
        success, msg = self.controller.remove_context_menu()
        if success:
            QMessageBox.information(self, tr("sidebar.integration.title"), msg)
        else:
            QMessageBox.warning(self, tr("sidebar.error.title"), msg)

    def _install_cli(self):
        self.on_settings_change()
        success, msg = self.controller.install_cli()
        if success:
            QMessageBox.information(self, tr("sidebar.cli_integration.title"), msg)
        else:
            QMessageBox.warning(self, tr("sidebar.error.title"), msg)

    def _remove_cli(self):
        success, msg = self.controller.remove_cli()
        if success:
            QMessageBox.information(self, tr("sidebar.cli_integration.title"), msg)
        else:
            QMessageBox.warning(self, tr("sidebar.error.title"), msg)

    def _open_patch_dialog(self):
        from ..dialogs import JsonPatchDialog, InteractiveDiffDialog
        dialog = JsonPatchDialog(self)
        if dialog.exec():
            json_str = dialog.get_json()
            if json_str:
                prepared_patches = self.controller.prepare_llm_patch(json_str)
                if prepared_patches:
                    diff_dialog = InteractiveDiffDialog(self, prepared_patches, self.controller)
                    if diff_dialog.exec():
                        selected = diff_dialog.get_selected()
                        if selected:
                            self.controller.apply_prepared_patches(selected)
                        else:
                            QMessageBox.warning(self, tr("sidebar.patch_error.title"), tr("sidebar.patch_error.message"))

    def _check_updates(self):
        self.on_settings_change()
        version = get_app_version()
        self.controller.check_for_updates(version)

    def _on_ext_preset_change(self, text):
        if not text: return
        custom = self.controller._store.state.settings.custom_presets
        if text in PRESETS:
            self.entry_ext.setPlainText(PRESETS[text]['ext'].replace(' ', '\n'))
            self.entry_ign.setPlainText(PRESETS[text]['ign'].replace(', ', '\n').replace(',', '\n'))
        elif text in custom:
            self.entry_ext.setPlainText(custom[text]['ext'].replace(' ', '\n'))
            self.entry_ign.setPlainText(custom[text]['ign'].replace(', ', '\n').replace(',', '\n'))
        self.on_settings_change()

    def _save_ext_preset(self):
        name, ok = QInputDialog.getText(self, tr("sidebar.new_preset.title"), tr("sidebar.new_preset.prompt"))
        if ok and name:
            if name in PRESETS:
                QMessageBox.warning(self, tr("sidebar.error.title"), tr("sidebar.new_preset.name_taken"))
                return
            ext = self.entry_ext.toPlainText().replace('\n', ' ').strip()
            ign = self.entry_ign.toPlainText().replace('\n', ', ').strip()

            custom = self.controller._store.state.settings.custom_presets.copy()
            custom[name] = {"ext": ext, "ign": ign}
            self.controller.update_settings({'custom_presets': custom})
            self.controller.save_settings()

            self._refresh_ext_presets()
            self.cmb_preset.setCurrentText(name)

    def _del_ext_preset(self):
        name = self.cmb_preset.currentText()
        if name in PRESETS:
            QMessageBox.warning(self, tr("sidebar.error.title"), tr("sidebar.delete_preset.system_preset"))
            return
        custom = self.controller._store.state.settings.custom_presets.copy()
        if name in custom:
            del custom[name]
            self.controller.update_settings({'custom_presets': custom})
            self.controller.save_settings()
            self._refresh_ext_presets()

    def _on_prompt_preset_change(self, text):
        if not text: return
        custom = self.controller._store.state.settings.custom_prompt_presets
        if text == "Custom":
            pass
        elif text in PROMPT_PRESETS:
            self.txt_system_prompt.setText(PROMPT_PRESETS[text])
        elif text in custom:
            self.txt_system_prompt.setText(custom[text])
        self.on_settings_change()

    def _save_prompt_preset(self):
        name, ok = QInputDialog.getText(self, tr("sidebar.new_prompt.title"), tr("sidebar.new_prompt.prompt"))
        if ok and name:
            if name in PROMPT_PRESETS or name == "Custom":
                QMessageBox.warning(self, tr("sidebar.error.title"), tr("sidebar.new_prompt.name_taken"))
                return
            txt = self.txt_system_prompt.toPlainText().strip()
            custom = self.controller._store.state.settings.custom_prompt_presets.copy()
            custom[name] = txt
            self.controller.update_settings({'custom_prompt_presets': custom})
            self.controller.save_settings()
            self._refresh_prompt_presets()
            self.cmb_prompt.setCurrentText(name)

    def _del_prompt_preset(self):
        name = self.cmb_prompt.currentText()
        if name in PROMPT_PRESETS or name == "Custom":
            QMessageBox.warning(self, tr("sidebar.error.title"), tr("sidebar.delete_prompt.system_preset"))
            return
        custom = self.controller._store.state.settings.custom_prompt_presets.copy()
        if name in custom:
            del custom[name]
            self.controller.update_settings({'custom_prompt_presets': custom})
            self.controller.save_settings()
            self._refresh_prompt_presets()

    def _get_user_themes_dir(self):
        from ...utils.config import get_app_data_dir
        path = os.path.join(get_app_data_dir(), "themes")
        os.makedirs(path, exist_ok=True)
        return path

    def _open_themes_folder(self):
        themes_dir = self._get_user_themes_dir()
        system = platform.system()
        try:
            if system == "Windows":
                os.startfile(themes_dir)
            elif system == "Darwin":
                import subprocess
                subprocess.call(["open", themes_dir])
            else:
                import subprocess
                subprocess.call(["xdg-open", themes_dir])
        except Exception as e:
            QMessageBox.warning(self, tr("sidebar.error.title"), tr("sidebar.open_folder.error", error=str(e)))

    def _import_theme(self):
        path, _ = QFileDialog.getOpenFileName(self, tr("sidebar.import_theme.title"), "", "JSON Files (*.json)")
        if path:
            themes_dir = self._get_user_themes_dir()
            filename = os.path.basename(path)
            dest = os.path.join(themes_dir, filename)
            try:
                shutil.copy2(path, dest)
                from ...utils.config import get_app_data_dir
                if getattr(sys, 'frozen', False):
                    built_in = os.path.join(sys._MEIPASS, "themes")
                else:
                    built_in = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))), "themes")
                ThemeManager.load_themes(built_in, themes_dir)
                self._refresh_themes()
                self.cmb_theme.setCurrentText(filename.replace(".json", ""))
                QMessageBox.information(self, tr("sidebar.success.title"), tr("sidebar.import_theme.success", filename=filename))
            except Exception as e:
                QMessageBox.warning(self, tr("sidebar.error.title"), tr("sidebar.import_theme.error", error=e))

    def _refresh_ext_presets(self):
        current = self.cmb_preset.currentText()
        self.cmb_preset.blockSignals(True)
        self.cmb_preset.clear()
        self.cmb_preset.addItems(list(PRESETS.keys()))
        custom = self.controller._store.state.settings.custom_presets
        if custom:
            self.cmb_preset.insertSeparator(self.cmb_preset.count())
            self.cmb_preset.addItems(list(custom.keys()))

        idx = self.cmb_preset.findText(current)
        if idx >= 0:
            self.cmb_preset.setCurrentIndex(idx)
        else:
            settings_ext = self.controller._store.state.settings.extensions
            settings_ign = self.controller._store.state.settings.ignored_paths
            matched = False
            for k, v in PRESETS.items():
                if v['ext'] == settings_ext and v['ign'] == settings_ign:
                    idx = self.cmb_preset.findText(k)
                    if idx >= 0:
                        self.cmb_preset.setCurrentIndex(idx)
                        matched = True
                        break
            if not matched and custom:
                for k, v in custom.items():
                    if v['ext'] == settings_ext and v['ign'] == settings_ign:
                        idx = self.cmb_preset.findText(k)
                        if idx >= 0:
                            self.cmb_preset.setCurrentIndex(idx)
                            matched = True
                            break
            if not matched:
                self.cmb_preset.setCurrentIndex(0)
        self.cmb_preset.blockSignals(False)

    def _refresh_prompt_presets(self):
        current = self.cmb_prompt.currentText()
        self.cmb_prompt.blockSignals(True)
        self.cmb_prompt.clear()
        self.cmb_prompt.addItems(list(PROMPT_PRESETS.keys()))
        custom = self.controller._store.state.settings.custom_prompt_presets
        if custom:
            self.cmb_prompt.insertSeparator(self.cmb_prompt.count())
            self.cmb_prompt.addItems(list(custom.keys()))

        idx = self.cmb_prompt.findText(current)
        if idx >= 0:
            self.cmb_prompt.setCurrentIndex(idx)
        else:
            self.cmb_prompt.setCurrentIndex(0)
        self.cmb_prompt.blockSignals(False)

    def _refresh_themes(self):
        current = self.cmb_theme.currentText()
        self.cmb_theme.blockSignals(True)
        self.cmb_theme.clear()
        themes = ThemeManager.get_available_themes()
        self.cmb_theme.addItems(themes)

        idx = self.cmb_theme.findText(current)
        if idx >= 0:
            self.cmb_theme.setCurrentIndex(idx)
        elif themes:
            self.cmb_theme.setCurrentIndex(0)
        self.cmb_theme.blockSignals(False)

    def update_ui(self, settings):
        visible_tabs = getattr(settings, 'visible_tabs', ["sources", "filters", "prompts", "llm_os", "appearance"])

        if getattr(self, '_current_visible_tabs', None) != visible_tabs:
            self._rebuild_tabs(visible_tabs)

        self.cmb_preset.blockSignals(True)
        self._refresh_ext_presets()
        self.cmb_preset.blockSignals(False)

        self.cmb_prompt.blockSignals(True)
        self._refresh_prompt_presets()
        self.cmb_prompt.blockSignals(False)

        self.cmb_theme.blockSignals(True)
        self._refresh_themes()
        self.cmb_theme.setCurrentText(ThemeManager._current_theme)
        self.cmb_theme.blockSignals(False)

        self.entry_ext.setPlainText(settings.extensions.replace(' ', '\n'))
        self.entry_ign.setPlainText(settings.ignored_paths.replace(', ', '\n').replace(',', '\n'))
        self.chk_tree.setChecked(settings.include_tree)
        self.chk_dependencies.setChecked(settings.include_dependencies)
        self.chk_mermaid.setChecked(getattr(settings, 'include_mermaid', False))
        self.chk_git.setChecked(settings.use_git)
        self.chk_gitignore.setChecked(settings.use_gitignore)
        self.txt_system_prompt.setText(settings.system_prompt)
        self.chk_llm_check.setChecked(settings.llm_check_enabled)
        self.entry_llm_url.setText(settings.llm_base_url)
        self.entry_llm_key.setText(settings.llm_api_key)
        self.entry_llm_model.setText(settings.llm_model)
        self.entry_editor.setText(getattr(settings, 'external_editor', ''))
        self.chk_prerelease.setChecked(settings.receive_prereleases)

    def get_settings(self):
        return {
            'extensions': self.entry_ext.toPlainText().replace('\n', ' ').strip(),
            'ignored_paths': self.entry_ign.toPlainText().replace('\n', ', ').strip(),
            'include_tree': self.chk_tree.isChecked(),
            'include_dependencies': self.chk_dependencies.isChecked(),
            'include_mermaid': self.chk_mermaid.isChecked(),
            'use_git': self.chk_git.isChecked(),
            'use_gitignore': self.chk_gitignore.isChecked(),
            'system_prompt': self.txt_system_prompt.toPlainText().strip(),
            'llm_check_enabled': self.chk_llm_check.isChecked(),
            'llm_base_url': self.entry_llm_url.text().strip(),
            'llm_api_key': self.entry_llm_key.text().strip(),
            'llm_model': self.entry_llm_model.text().strip(),
            'external_editor': self.entry_editor.text().strip(),
            'receive_prereleases': self.chk_prerelease.isChecked()
        }
