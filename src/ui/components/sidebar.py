from PySide6.QtWidgets import (QWidget, QVBoxLayout, QTabWidget, QFormLayout,
                               QCheckBox, QComboBox, QLineEdit, QTextEdit,
                               QPushButton, QHBoxLayout, QLabel, QFileDialog,
                               QInputDialog, QMessageBox)
from PySide6.QtCore import Qt
from ...utils.config import PRESETS, PROMPT_PRESETS
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

        self.tab_run = QWidget()
        self.tab_prompt = QWidget()
        self.tab_settings = QWidget()

        self.tabs.addTab(self.tab_run, "Сборка")
        self.tabs.addTab(self.tab_prompt, "Промпты")
        self.tabs.addTab(self.tab_settings, "Настройки")

        self._build_run_tab()
        self._build_prompt_tab()
        self._build_settings_tab()

        bottom_layout = QHBoxLayout()
        btn_tour = QPushButton("Инструкция")
        btn_tour.setProperty("cssClass", "ghost")
        btn_tour.clicked.connect(self.controller.show_tour)

        lbl_version = QLabel("v6.0 Qt Edition")
        lbl_version.setProperty("cssClass", "muted")

        bottom_layout.addWidget(lbl_version)
        bottom_layout.addStretch()
        bottom_layout.addWidget(btn_tour)

        self.layout.addLayout(bottom_layout)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setContentsMargins(m, m, m, m)
        self.layout.setSpacing(s)

    def _build_run_tab(self):
        layout = QVBoxLayout(self.tab_run)
        layout.setContentsMargins(0, 10, 0, 0)

        self.cmb_preset = QComboBox()
        self.cmb_preset.addItems(list(PRESETS.keys()))
        self.cmb_preset.currentTextChanged.connect(self.controller.apply_preset)

        self.entry_ext = QLineEdit()
        self.entry_ign = QLineEdit()

        self.chk_git = QCheckBox("Только Git Changes")
        self.chk_gitignore = QCheckBox("Учитывать .gitignore")
        self.chk_tree = QCheckBox("Дерево файлов")
        self.chk_dependencies = QCheckBox("Карта зависимостей")

        form = QFormLayout()
        form.setContentsMargins(0, 0, 0, 0)
        form.addRow("Пресет:", self.cmb_preset)
        form.addRow("Расширения:", self.entry_ext)
        form.addRow("Игнор:", self.entry_ign)
        layout.addLayout(form)

        layout.addWidget(self.chk_git)
        layout.addWidget(self.chk_gitignore)
        layout.addWidget(self.chk_tree)
        layout.addWidget(self.chk_dependencies)

        btn_layout = QHBoxLayout()
        btn_layout.setContentsMargins(0, 10, 0, 10)

        btn_add = QPushButton("+ Папка")
        btn_add.setProperty("cssClass", "ghost")
        btn_add.clicked.connect(self._add_folder)

        btn_gh = QPushButton("+ GitHub")
        btn_gh.setProperty("cssClass", "success")
        btn_gh.clicked.connect(self._add_github)

        btn_layout.addWidget(btn_add)
        btn_layout.addWidget(btn_gh)
        layout.addLayout(btn_layout)

        btn_scan = QPushButton("🔍 Сканировать (Preview)")
        btn_scan.clicked.connect(self._trigger_scan)
        layout.addWidget(btn_scan)

        btn_clear = QPushButton("Очистить")
        btn_clear.setProperty("cssClass", "ghost")
        btn_clear.clicked.connect(self.controller.clear_folders)
        layout.addWidget(btn_clear)

        layout.addStretch()

    def _add_folder(self):
        path = QFileDialog.getExistingDirectory(self, "Выберите папку")
        if path:
            self.controller.add_folder(path)

    def _add_github(self):
        url, ok = QInputDialog.getText(self, "GitHub",
                                       "Введите URL репозитория (например, https://github.com/user/repo):")
        if ok and url:
            reply = QMessageBox.question(
                self,
                "Сохранение",
                "Сохранить репозиторий на диск навсегда?\n\n"
                "• Да — выбрать папку на ПК\n"
                "• Нет — загрузить во временную папку (удалится при закрытии)",
                QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel
            )

            if reply == QMessageBox.Yes:
                dest_dir = QFileDialog.getExistingDirectory(self,
                                                            "Выберите папку для клонирования (туда будет создана папка проекта)")
                if dest_dir:
                    self.controller.add_github_repo(url, dest_dir)
            elif reply == QMessageBox.No:
                self.controller.add_github_repo(url, "")

    def _trigger_scan(self):
        self.on_settings_change()
        self.controller.scan_only()

    def _build_prompt_tab(self):
        layout = QVBoxLayout(self.tab_prompt)
        layout.setContentsMargins(0, 10, 0, 0)

        self.cmb_prompt = QComboBox()
        self.cmb_prompt.addItems(list(PROMPT_PRESETS.keys()))

        self.txt_system_prompt = QTextEdit()

        layout.addWidget(QLabel("Пресет промпта:"))
        layout.addWidget(self.cmb_prompt)
        layout.addWidget(QLabel("Системный промпт:"))
        layout.addWidget(self.txt_system_prompt)

        self.cmb_prompt.currentTextChanged.connect(self._on_prompt_preset_change)

        btn_patch = QPushButton("🧩 Применить JSON-патч от LLM")
        btn_patch.setProperty("cssClass", "success")
        btn_patch.clicked.connect(self._open_patch_dialog)
        layout.addWidget(btn_patch)

    def _open_patch_dialog(self):
        from ..dialogs import JsonPatchDialog, InteractiveDiffDialog

        # Шаг 1: Получаем JSON
        dialog = JsonPatchDialog(self)
        if dialog.exec():
            json_str = dialog.get_json()
            if json_str:
                # Шаг 2: Генерируем Diff'ы в памяти
                prepared_patches = self.controller.prepare_llm_patch(json_str)
                if prepared_patches:
                    # Шаг 3: Открываем окно предпросмотра Diff Viewer
                    diff_dialog = InteractiveDiffDialog(self, prepared_patches, self.controller)
                    if diff_dialog.exec():
                        selected = diff_dialog.get_selected()
                        if selected:
                            self.controller.apply_prepared_patches(selected)

    def _on_prompt_preset_change(self, text):
        if text != "Custom" and text in PROMPT_PRESETS:
            self.txt_system_prompt.setText(PROMPT_PRESETS[text])
            self.on_settings_change()

    def _build_settings_tab(self):
        layout = QVBoxLayout(self.tab_settings)
        layout.setContentsMargins(0, 10, 0, 0)

        self.cmb_theme = QComboBox()
        themes = ThemeManager.get_available_themes()
        self.cmb_theme.addItems(themes)
        self.cmb_theme.setCurrentText(ThemeManager._current_theme)
        self.cmb_theme.currentTextChanged.connect(lambda t: ThemeManager.apply_theme(theme_id=t))

        self.cmb_mode = QComboBox()
        self.cmb_mode.addItems(["light", "dark"])
        self.cmb_mode.setCurrentText(ThemeManager._current_mode)
        self.cmb_mode.currentTextChanged.connect(lambda m: ThemeManager.apply_theme(mode=m))

        # Настройки LLM Validator
        self.chk_llm_check = QCheckBox("Включить проверку кода (LLM Checker)")
        self.entry_llm_url = QLineEdit()
        self.entry_llm_url.setPlaceholderText("https://api.openai.com/v1")
        self.entry_llm_key = QLineEdit()
        self.entry_llm_key.setEchoMode(QLineEdit.Password)
        self.entry_llm_key.setPlaceholderText("sk-...")
        self.entry_llm_model = QLineEdit()
        self.entry_llm_model.setPlaceholderText("gpt-4o-mini / local-model")

        # CLI
        self.chk_cli_minify = QCheckBox("CLI Minify")
        self.chk_cli_comments = QCheckBox("CLI Убрать комментарии")
        self.chk_cli_tree = QCheckBox("CLI Включать дерево")
        self.cmb_cli_format = QComboBox()
        self.cmb_cli_format.addItems(["plain", "markdown", "xml"])

        form = QFormLayout()
        form.setContentsMargins(0, 10, 0, 0)
        form.addRow("Тема:", self.cmb_theme)
        form.addRow("Режим:", self.cmb_mode)

        lbl_llm = QLabel("LLM Validator (Настройки API):")
        lbl_llm.setProperty("cssClass", "muted")
        layout.addLayout(form)
        layout.addWidget(lbl_llm)
        layout.addWidget(self.chk_llm_check)

        form_llm = QFormLayout()
        form_llm.setContentsMargins(0, 0, 0, 0)
        form_llm.addRow("URL:", self.entry_llm_url)
        form_llm.addRow("Ключ:", self.entry_llm_key)
        form_llm.addRow("Модель:", self.entry_llm_model)
        layout.addLayout(form_llm)

        lbl_cli = QLabel("CLI Настройки:")
        lbl_cli.setProperty("cssClass", "muted")
        layout.addWidget(lbl_cli)

        layout.addWidget(self.chk_cli_minify)
        layout.addWidget(self.chk_cli_comments)
        layout.addWidget(self.chk_cli_tree)

        form_cli = QFormLayout()
        form_cli.setContentsMargins(0, 0, 0, 0)
        form_cli.addRow("CLI Формат:", self.cmb_cli_format)
        layout.addLayout(form_cli)

        layout.addStretch()

    def update_ui(self, settings):
        self.entry_ext.setText(settings.extensions)
        self.entry_ign.setText(settings.ignored_paths)
        self.chk_tree.setChecked(settings.include_tree)
        self.chk_dependencies.setChecked(settings.include_dependencies)
        self.chk_git.setChecked(settings.use_git)
        self.chk_gitignore.setChecked(settings.use_gitignore)
        self.txt_system_prompt.setText(settings.system_prompt)

        # Обновление полей LLM
        self.chk_llm_check.setChecked(settings.llm_check_enabled)
        self.entry_llm_url.setText(settings.llm_base_url)
        self.entry_llm_key.setText(settings.llm_api_key)
        self.entry_llm_model.setText(settings.llm_model)

        self.chk_cli_minify.setChecked(settings.cli_minify)
        self.chk_cli_comments.setChecked(settings.cli_remove_comments)
        self.chk_cli_tree.setChecked(settings.cli_include_tree)
        self.cmb_cli_format.setCurrentText(settings.cli_format)

    def get_settings(self):
        return {
            'extensions': self.entry_ext.text(),
            'ignored_paths': self.entry_ign.text(),
            'include_tree': self.chk_tree.isChecked(),
            'include_dependencies': self.chk_dependencies.isChecked(),
            'use_git': self.chk_git.isChecked(),
            'use_gitignore': self.chk_gitignore.isChecked(),
            'system_prompt': self.txt_system_prompt.toPlainText(),

            'llm_check_enabled': self.chk_llm_check.isChecked(),
            'llm_base_url': self.entry_llm_url.text(),
            'llm_api_key': self.entry_llm_key.text(),
            'llm_model': self.entry_llm_model.text(),

            'cli_minify': self.chk_cli_minify.isChecked(),
            'cli_remove_comments': self.chk_cli_comments.isChecked(),
            'cli_include_tree': self.chk_cli_tree.isChecked(),
            'cli_format': self.cmb_cli_format.currentText(),
        }