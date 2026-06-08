from PySide6.QtWidgets import QWidget, QHBoxLayout, QVBoxLayout, QCheckBox, QComboBox, QPushButton, QFileDialog
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus


class ActionPanel(QWidget):
    def __init__(self, on_run_callback):
        super().__init__()
        self.on_run = on_run_callback
        self._current_template_path = ""

        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setProperty("cssClass", "card")

        self.layout = QVBoxLayout(self)
        self.layout.setSpacing(4)

        # Row 1: checkboxes + format
        row1 = QHBoxLayout()
        row1.setSpacing(8)

        self.chk_minify = QCheckBox("Minify")
        self.chk_comments = QCheckBox("No Comments")
        self.chk_secrets = QCheckBox("No Secrets")
        self.chk_skeleton = QCheckBox("Skeleton")

        self.cmb_format = QComboBox()
        self.cmb_format.addItems(["markdown", "xml", "plain", "jsonl_chunk", "custom"])
        self.cmb_format.currentTextChanged.connect(self._on_format_changed)

        self.btn_template = QPushButton("📁")
        self.btn_template.setToolTip("Выбрать Jinja2 шаблон")
        self.btn_template.setProperty("cssClass", "icon")
        self.btn_template.clicked.connect(self._pick_template)
        self.btn_template.hide()

        row1.addWidget(self.chk_minify)
        row1.addWidget(self.chk_comments)
        row1.addWidget(self.chk_secrets)
        row1.addWidget(self.chk_skeleton)
        row1.addWidget(self.cmb_format)
        row1.addWidget(self.btn_template)
        row1.addStretch()

        # Row 2: action buttons
        row2 = QHBoxLayout()
        row2.setSpacing(8)

        btn_preview = QPushButton("👀 Предпросмотр")
        btn_preview.setProperty("cssClass", "ghost")
        btn_preview.clicked.connect(lambda: self.on_run("preview"))

        btn_copy = QPushButton("📋 В Буфер обмена")
        btn_copy.clicked.connect(lambda: self.on_run("clipboard"))

        self.btn_chat = QPushButton("🚀 Отправить в ChatGPT / Claude")
        self.btn_chat.setProperty("cssClass", "success")
        self.btn_chat.clicked.connect(lambda: self.on_run("chat"))

        btn_editor = QPushButton("💻 В редактор")
        btn_editor.setProperty("cssClass", "ghost")
        btn_editor.setToolTip("Открыть результат в VS Code / Cursor")
        btn_editor.clicked.connect(lambda: self.on_run("editor"))

        btn_file = QPushButton("💾 В Файл")
        btn_file.setProperty("cssClass", "ghost")
        btn_file.clicked.connect(lambda: self.on_run("file"))

        row2.addWidget(btn_preview)
        row2.addWidget(btn_copy)
        row2.addWidget(btn_editor)
        row2.addWidget(self.btn_chat)
        row2.addWidget(btn_file)
        row2.addStretch()

        self.layout.addLayout(row1)
        self.layout.addLayout(row2)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setContentsMargins(m, 4, m, 4)
        self.layout.setSpacing(4)

    def _on_format_changed(self, fmt):
        self.btn_template.setVisible(fmt == "custom")

    def _pick_template(self):
        path, _ = QFileDialog.getOpenFileName(
            self, "Выберите Jinja2 шаблон", "", "Jinja2 Templates (*.jinja2 *.j2 *.html);;All Files (*.*)"
        )
        if path:
            self._current_template_path = path

    def update_ui(self, settings):
        self.chk_minify.setChecked(settings.minify)
        self.chk_comments.setChecked(settings.remove_comments)
        self.chk_secrets.setChecked(settings.remove_secrets)
        self.chk_skeleton.setChecked(settings.skeleton_mode)

        # Восстанавливаем сохраненный путь
        self._current_template_path = settings.template_path
        self.cmb_format.setCurrentText(settings.output_format)
        self.btn_template.setVisible(settings.output_format == "custom")

    def get_settings(self):
        return {
            'minify': self.chk_minify.isChecked(),
            'remove_comments': self.chk_comments.isChecked(),
            'remove_secrets': self.chk_secrets.isChecked(),
            'skeleton_mode': self.chk_skeleton.isChecked(),
            'output_format': self.cmb_format.currentText(),
            'template_path': getattr(self, '_current_template_path', "")
        }