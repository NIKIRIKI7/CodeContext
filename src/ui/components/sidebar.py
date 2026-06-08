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


class Sidebar(QWidget):
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

        self.tab_sources = QWidget()
        self.tab_filters = QWidget()
        self.tab_prompts = QWidget()
        self.tab_llm_os = QWidget()
        self.tab_appearance = QWidget()

        self.tabs.addTab(self.tab_sources, "Источники")
        self.tabs.addTab(self.tab_filters, "Фильтры")
        self.tabs.addTab(self.tab_prompts, "Промпты")
        self.tabs.addTab(self.tab_llm_os, "LLM & ОС")
        self.tabs.addTab(self.tab_appearance, "Темы")

        self._build_sources_tab()
        self._build_filters_tab()
        self._build_prompts_tab()
        self._build_llm_os_tab()
        self._build_appearance_tab()

        bottom_layout = QHBoxLayout()
        btn_tour = QPushButton("Инструкция")
        btn_tour.setProperty("cssClass", "ghost")
        btn_tour.clicked.connect(self.controller.show_tour)

        version_str = get_app_version()
        lbl_version = QLabel(f"v{version_str}")
        lbl_version.setProperty("cssClass", "muted")

        btn_update = QPushButton("🔄 Обновления")
        btn_update.setProperty("cssClass", "ghost")
        btn_update.clicked.connect(self._check_updates)

        bottom_layout.addWidget(lbl_version)
        bottom_layout.addStretch()
        bottom_layout.addWidget(btn_update)
        bottom_layout.addWidget(btn_tour)
        self.layout.addLayout(bottom_layout)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setContentsMargins(m, m, m, m)
        self.layout.setSpacing(s)

    def _build_sources_tab(self):
        layout = QVBoxLayout(self.tab_sources)
        layout.setContentsMargins(0, 10, 0, 0)

        btn_layout = QHBoxLayout()
        btn_layout.setContentsMargins(0, 0, 0, 10)
        btn_add = QPushButton("+ Папка ПК")
        btn_add.setProperty("cssClass", "ghost")
        btn_add.clicked.connect(self._add_folder)
        btn_gh = QPushButton("+ GitHub")
        btn_gh.setProperty("cssClass", "success")
        btn_gh.clicked.connect(self._add_github)
        btn_layout.addWidget(btn_add)
        btn_layout.addWidget(btn_gh)
        layout.addLayout(btn_layout)

        self.chk_git = QCheckBox("Только Git Changes")
        self.chk_gitignore = QCheckBox("Учитывать .gitignore")
        layout.addWidget(self.chk_git)
        layout.addWidget(self.chk_gitignore)

        layout.addSpacing(10)
        btn_scan = QPushButton("🔍 Сканировать файлы")
        btn_scan.clicked.connect(self._trigger_scan)
        layout.addWidget(btn_scan)

        btn_clear = QPushButton("Очистить проект")
        btn_clear.setProperty("cssClass", "ghost")
        btn_clear.clicked.connect(self.controller.clear_folders)
        layout.addWidget(btn_clear)
        layout.addStretch()

    def _build_filters_tab(self):
        layout = QVBoxLayout(self.tab_filters)
        layout.setContentsMargins(0, 10, 0, 0)

        preset_layout = QHBoxLayout()
        self.cmb_preset = QComboBox()
        self.cmb_preset.setSizeAdjustPolicy(QComboBox.AdjustToContents)
        self.cmb_preset.currentTextChanged.connect(self._on_ext_preset_change)

        btn_save_preset = QPushButton("💾")
        btn_save_preset.setProperty("cssClass", "icon")
        btn_save_preset.setToolTip("Сохранить как пресет")
        btn_save_preset.clicked.connect(self._save_ext_preset)

        btn_del_preset = QPushButton("🗑")
        btn_del_preset.setProperty("cssClass", "icon")
        btn_del_preset.setToolTip("Удалить выбранный кастомный пресет")
        btn_del_preset.clicked.connect(self._del_ext_preset)

        preset_layout.addWidget(QLabel("Пресет:"))
        preset_layout.addWidget(self.cmb_preset, 1)
        preset_layout.addWidget(btn_save_preset)
        preset_layout.addWidget(btn_del_preset)
        layout.addLayout(preset_layout)

        layout.addWidget(QLabel("Расширения:"))
        self.entry_ext = QPlainTextEdit()
        self.entry_ext.setProperty("cssClass", "textarea_small")
        self.entry_ext.setPlaceholderText("Например: .py .js .ts")
        layout.addWidget(self.entry_ext)

        layout.addWidget(QLabel("Игнорировать пути:"))
        self.entry_ign = QPlainTextEdit()
        self.entry_ign.setProperty("cssClass", "textarea_small")
        self.entry_ign.setPlaceholderText("Например: node_modules, .git, build")
        layout.addWidget(self.entry_ign)

        self.chk_tree = QCheckBox("Включить дерево файлов")
        self.chk_dependencies = QCheckBox("Включить карту зависимостей")
        layout.addWidget(self.chk_tree)
        layout.addWidget(self.chk_dependencies)
        layout.addStretch()

    def _build_prompts_tab(self):
        layout = QVBoxLayout(self.tab_prompts)
        layout.setContentsMargins(0, 10, 0, 0)

        preset_layout = QHBoxLayout()
        self.cmb_prompt = QComboBox()
        self.cmb_prompt.currentTextChanged.connect(self._on_prompt_preset_change)

        btn_save_prompt = QPushButton("💾")
        btn_save_prompt.setProperty("cssClass", "icon")
        btn_save_prompt.setToolTip("Сохранить как пресет")
        btn_save_prompt.clicked.connect(self._save_prompt_preset)

        btn_del_prompt = QPushButton("🗑")
        btn_del_prompt.setProperty("cssClass", "icon")
        btn_del_prompt.setToolTip("Удалить выбранный кастомный пресет")
        btn_del_prompt.clicked.connect(self._del_prompt_preset)

        preset_layout.addWidget(QLabel("Пресет:"))
        preset_layout.addWidget(self.cmb_prompt, 1)
        preset_layout.addWidget(btn_save_prompt)
        preset_layout.addWidget(btn_del_prompt)
        layout.addLayout(preset_layout)

        layout.addWidget(QLabel("Системный промпт:"))
        self.txt_system_prompt = QTextEdit()
        layout.addWidget(self.txt_system_prompt)

        btn_patch = QPushButton("🧩 Применить JSON-патч от LLM")
        btn_patch.setProperty("cssClass", "success")
        btn_patch.clicked.connect(self._open_patch_dialog)
        layout.addWidget(btn_patch)

    def _build_llm_os_tab(self):
        layout = QVBoxLayout(self.tab_llm_os)
        layout.setContentsMargins(0, 10, 0, 0)

        lbl_llm = QLabel("LLM Validator (API):")
        lbl_llm.setProperty("cssClass", "heading")
        layout.addWidget(lbl_llm)

        self.chk_llm_check = QCheckBox("Включить проверку (LLM Checker)")
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

        form_llm.addRow("URL:", self.entry_llm_url)
        form_llm.addRow("Ключ:", self.entry_llm_key)
        form_llm.addRow("Модель:", self.entry_llm_model)
        layout.addLayout(form_llm)

        layout.addSpacing(10)
        lbl_os = QLabel(f"Интеграция с ОС ({platform.system()}):")
        lbl_os.setProperty("cssClass", "heading")
        layout.addWidget(lbl_os)

        btn_ctx_layout = QHBoxLayout()
        self.btn_install_ctx = QPushButton("В меню проводника")
        self.btn_install_ctx.setProperty("cssClass", "success")
        self.btn_install_ctx.clicked.connect(self._install_context_menu)

        self.btn_remove_ctx = QPushButton("Удалить из меню")
        self.btn_remove_ctx.setProperty("cssClass", "ghost")
        self.btn_remove_ctx.clicked.connect(self._remove_context_menu)

        btn_ctx_layout.addWidget(self.btn_install_ctx)
        btn_ctx_layout.addWidget(self.btn_remove_ctx)
        layout.addLayout(btn_ctx_layout)

        layout.addSpacing(10)
        lbl_upd = QLabel("Настройки обновления:")
        lbl_upd.setProperty("cssClass", "heading")
        layout.addWidget(lbl_upd)
        self.chk_prerelease = QCheckBox("Получать Pre-release версии")
        layout.addWidget(self.chk_prerelease)

        layout.addStretch()

    def _build_appearance_tab(self):
        layout = QVBoxLayout(self.tab_appearance)
        layout.setContentsMargins(0, 10, 0, 0)

        form = QFormLayout()
        form.setContentsMargins(0, 0, 0, 0)

        self.cmb_theme = QComboBox()
        self.cmb_theme.currentTextChanged.connect(lambda t: ThemeManager.apply_theme(theme_id=t))

        self.cmb_mode = QComboBox()
        self.cmb_mode.addItems(["light", "dark"])
        self.cmb_mode.setCurrentText(ThemeManager._current_mode)
        self.cmb_mode.currentTextChanged.connect(lambda m: ThemeManager.apply_theme(mode=m))

        form.addRow("Тема:", self.cmb_theme)
        form.addRow("Режим:", self.cmb_mode)
        layout.addLayout(form)

        layout.addSpacing(20)
        btn_themes_folder = QPushButton("📂 Открыть папку тем")
        btn_themes_folder.setProperty("cssClass", "ghost")
        btn_themes_folder.clicked.connect(self._open_themes_folder)

        btn_import_theme = QPushButton("➕ Импортировать тему (.json)")
        btn_import_theme.setProperty("cssClass", "success")
        btn_import_theme.clicked.connect(self._import_theme)

        layout.addWidget(btn_themes_folder)
        layout.addWidget(btn_import_theme)
        layout.addStretch()

    def _add_folder(self):
        path = QFileDialog.getExistingDirectory(self, "Выберите папку")
        if path:
            self.controller.add_folder(path)

    def _add_github(self):
        url, ok = QInputDialog.getText(self, "GitHub", "Введите URL репозитория (например, https://github.com/user/repo):")
        if ok and url:
            reply = QMessageBox.question(
                self, "Сохранение",
                "Сохранить репозиторий на диск навсегда?\n\n"
                "• Да — выбрать папку на ПК\n"
                "• Нет — загрузить во временную папку (удалится при закрытии)",
                QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel
            )
            if reply == QMessageBox.Yes:
                dest_dir = QFileDialog.getExistingDirectory(self, "Выберите папку для клонирования")
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
        if success or "запрошены права" in msg.lower():
            QMessageBox.information(self, "Интеграция", msg)
        else:
            QMessageBox.warning(self, "Ошибка", msg)

    def _remove_context_menu(self):
        success, msg = self.controller.remove_context_menu()
        if success or "запрошены права" in msg.lower():
            QMessageBox.information(self, "Интеграция", msg)
        else:
            QMessageBox.warning(self, "Ошибка", msg)

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
                            QMessageBox.warning(self, "Ошибка патча", "Не найдено валидных JSON-инструкций.\n\nУбедитесь, что ответ содержит массив объектов.")

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
        name, ok = QInputDialog.getText(self, "Новый пресет", "Введите имя пресета:")
        if ok and name:
            if name in PRESETS:
                QMessageBox.warning(self, "Ошибка", "Это имя занято системным пресетом.")
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
            QMessageBox.warning(self, "Ошибка", "Системные пресеты нельзя удалить.")
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
        name, ok = QInputDialog.getText(self, "Новый промпт", "Введите имя пресета:")
        if ok and name:
            if name in PROMPT_PRESETS or name == "Custom":
                QMessageBox.warning(self, "Ошибка", "Это имя занято системным пресетом.")
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
            QMessageBox.warning(self, "Ошибка", "Системные пресеты нельзя удалить.")
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
            QMessageBox.warning(self, "Ошибка", f"Не удалось открыть папку: {e}")

    def _import_theme(self):
        path, _ = QFileDialog.getOpenFileName(self, "Импорт темы", "", "JSON Files (*.json)")
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
                QMessageBox.information(self, "Успех", f"Тема {filename} импортирована!")
            except Exception as e:
                QMessageBox.warning(self, "Ошибка", f"Не удалось импортировать тему:\n{e}")

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
        self.chk_git.setChecked(settings.use_git)
        self.chk_gitignore.setChecked(settings.use_gitignore)
        self.txt_system_prompt.setText(settings.system_prompt)
        self.chk_llm_check.setChecked(settings.llm_check_enabled)
        self.entry_llm_url.setText(settings.llm_base_url)
        self.entry_llm_key.setText(settings.llm_api_key)
        self.entry_llm_model.setText(settings.llm_model)
        self.chk_prerelease.setChecked(settings.receive_prereleases)

    def get_settings(self):
        return {
            'extensions': self.entry_ext.toPlainText().replace('\n', ' ').strip(),
            'ignored_paths': self.entry_ign.toPlainText().replace('\n', ', ').strip(),
            'include_tree': self.chk_tree.isChecked(),
            'include_dependencies': self.chk_dependencies.isChecked(),
            'use_git': self.chk_git.isChecked(),
            'use_gitignore': self.chk_gitignore.isChecked(),
            'system_prompt': self.txt_system_prompt.toPlainText().strip(),
            'llm_check_enabled': self.chk_llm_check.isChecked(),
            'llm_base_url': self.entry_llm_url.text().strip(),
            'llm_api_key': self.entry_llm_key.text().strip(),
            'llm_model': self.entry_llm_model.text().strip(),
            'receive_prereleases': self.chk_prerelease.isChecked()
        }
