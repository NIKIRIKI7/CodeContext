from PySide6.QtWidgets import QWidget, QHBoxLayout, QCheckBox, QComboBox, QPushButton, QFileDialog
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus


class ActionPanel(QWidget):
    def __init__(self, on_run_callback):
        super().__init__()
        self.on_run = on_run_callback
        self._current_template_path = ""

        # Разрешаем кастомному QWidget отрисовывать QSS фоны
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setProperty("cssClass", "card")

        self.layout = QHBoxLayout(self)

        self.chk_minify = QCheckBox("Minify")
        self.chk_comments = QCheckBox("No Comments")
        self.chk_secrets = QCheckBox("No Secrets")
        self.chk_skeleton = QCheckBox("Skeleton")

        self.cmb_format = QComboBox()
        self.cmb_format.addItems(["markdown", "xml", "plain", "custom"])
        self.cmb_format.currentTextChanged.connect(self._on_format_changed)

        # Кнопка для выбора шаблона (показывается только при custom)
        self.btn_template = QPushButton("📁")
        self.btn_template.setToolTip("Выбрать Jinja2 шаблон")
        self.btn_template.setProperty("cssClass", "icon")
        self.btn_template.setFixedSize(28, 28)
        self.btn_template.clicked.connect(self._pick_template)
        self.btn_template.hide()

        format_layout = QHBoxLayout()
        format_layout.setContentsMargins(0, 0, 0, 0)
        format_layout.setSpacing(4)
        format_layout.addWidget(self.cmb_format)
        format_layout.addWidget(self.btn_template)

        btn_preview = QPushButton("👀 Предпросмотр")
        btn_preview.setProperty("cssClass", "ghost")
        btn_preview.clicked.connect(lambda: self.on_run("preview"))

        btn_copy = QPushButton("📋 В Буфер обмена")
        btn_copy.clicked.connect(lambda: self.on_run("clipboard"))

        btn_file = QPushButton("💾 Сохранить в Файл")
        btn_file.setProperty("cssClass", "ghost")
        btn_file.clicked.connect(lambda: self.on_run("file"))

        self.layout.addWidget(self.chk_minify)
        self.layout.addWidget(self.chk_comments)
        self.layout.addWidget(self.chk_secrets)
        self.layout.addWidget(self.chk_skeleton)
        self.layout.addLayout(format_layout)

        self.layout.addStretch()

        self.layout.addWidget(btn_preview)
        self.layout.addWidget(btn_copy)
        self.layout.addWidget(btn_file)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setContentsMargins(m, int(m / 2), m, int(m / 2))
        self.layout.setSpacing(s)

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