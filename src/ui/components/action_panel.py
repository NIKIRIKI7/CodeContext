from PySide6.QtWidgets import QWidget, QHBoxLayout, QVBoxLayout, QCheckBox, QComboBox, QPushButton, QFileDialog, QMenu
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus
from src.i18n import tr


class ActionPanel(QWidget):
    @staticmethod
    def _plugin_action_buttons(plugin_api):
        if plugin_api is None:
            return {}
        ui = getattr(plugin_api, 'ui', None)
        if ui is None:
            return {}
        return getattr(ui, 'action_buttons', {})

    def __init__(self, on_run_callback, plugin_api=None):
        super().__init__()
        self.on_run = on_run_callback
        self._current_template_path = ""
        self._plugin_api = plugin_api

        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setProperty("cssClass", "card")

        self.layout = QVBoxLayout(self)
        self.layout.setSpacing(4)

        row1 = QHBoxLayout()
        row1.setSpacing(8)

        self.chk_minify = QCheckBox(tr("action_panel.minify"))
        self.chk_comments = QCheckBox(tr("action_panel.no_comments"))
        self.chk_secrets = QCheckBox(tr("action_panel.no_secrets"))
        self.chk_skeleton = QCheckBox(tr("action_panel.skeleton"))

        self.cmb_format = QComboBox()
        self.cmb_format.addItems(["markdown", "xml", "plain", "jsonl_chunk", "custom"])
        self.cmb_format.currentTextChanged.connect(self._on_format_changed)

        self.btn_template = QPushButton("\U0001F4C1")
        self.btn_template.setToolTip(tr("action_panel.template.tooltip"))
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

        row_opt = QHBoxLayout()
        row_opt.setSpacing(8)
        self.chk_dedup = QCheckBox(tr("action_panel.chk_dedup"))
        self.chk_aggressive = QCheckBox(tr("action_panel.chk_aggressive"))
        self.chk_checkpoints = QCheckBox(tr("action_panel.chk_checkpoints"))
        self.chk_watch = QCheckBox(tr("action_panel.chk_watch"))
        row_opt.addWidget(self.chk_dedup)
        row_opt.addWidget(self.chk_aggressive)
        row_opt.addWidget(self.chk_checkpoints)
        row_opt.addWidget(self.chk_watch)
        row_opt.addStretch()
        self.layout.addLayout(row_opt)

        self.checkbox_buttons = {
            "dedup": self.chk_dedup,
            "aggressive": self.chk_aggressive,
            "checkpoints": self.chk_checkpoints,
            "watch": self.chk_watch,
        }

        row2 = QHBoxLayout()
        row2.setSpacing(8)

        self.btn_preview = QPushButton(tr("action_panel.preview.button"))
        self.btn_preview.setProperty("cssClass", "ghost")
        self.btn_preview.clicked.connect(lambda: self.on_run("preview"))

        self.btn_copy = QPushButton(tr("action_panel.clipboard.button"))
        self.btn_copy.clicked.connect(lambda: self.on_run("clipboard"))

        self.btn_chat = QPushButton(tr("action_panel.chat.button"))
        self.btn_chat.setProperty("cssClass", "success")
        self.btn_chat.clicked.connect(lambda: self.on_run("chat"))

        self.btn_editor = QPushButton(tr("action_panel.editor.button"))
        self.btn_editor.setProperty("cssClass", "ghost")
        self.btn_editor.setToolTip(tr("action_panel.editor.tooltip"))
        self.btn_editor.clicked.connect(lambda: self.on_run("editor"))

        self.btn_file = QPushButton(tr("action_panel.file.button"))
        self.btn_file.setProperty("cssClass", "ghost")
        self.btn_file.clicked.connect(lambda: self.on_run("file"))

        self.action_buttons = {
            "preview": self.btn_preview,
            "clipboard": self.btn_copy,
            "chat": self.btn_chat,
            "editor": self.btn_editor,
            "file": self.btn_file,
        }

        row2.addWidget(self.btn_preview)
        row2.addWidget(self.btn_copy)
        row2.addWidget(self.btn_editor)
        row2.addWidget(self.btn_chat)
        row2.addWidget(self.btn_file)

        plugin_actions = self._plugin_action_buttons(self._plugin_api)
        if plugin_actions:
            self.btn_plugins = QPushButton(tr("action_panel.plugins.button"))
            self.btn_plugins.setProperty("cssClass", "ghost")
            self._plugin_menu = QMenu()
            for a_id, pa in plugin_actions.items():
                action = self._plugin_menu.addAction(pa.get("label", a_id))
                cb = pa.get("callback")
                if cb:
                    action.triggered.connect(cb)
            self.btn_plugins.setMenu(self._plugin_menu)
            row2.addWidget(self.btn_plugins)

        row2.addStretch()

        self.layout.addLayout(row1)
        self.layout.addLayout(row2)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        self.layout.setContentsMargins(m, 4, m, 4)
        self.layout.setSpacing(4)

    def _on_format_changed(self, fmt):
        self.btn_template.setVisible(fmt == "custom")

    def _pick_template(self):
        path, _ = QFileDialog.getOpenFileName(
            self, tr("action_panel.template.dialog_title"), "", "Jinja2 Templates (*.jinja2 *.j2 *.html);;All Files (*.*)"
        )
        if path:
            self._current_template_path = path

    def update_ui(self, settings):
        self.chk_minify.setChecked(settings.minify)
        self.chk_comments.setChecked(settings.remove_comments)
        self.chk_secrets.setChecked(settings.remove_secrets)
        self.chk_skeleton.setChecked(settings.skeleton_mode)
        self.chk_dedup.setChecked(getattr(settings, 'deduplicate', False))
        self.chk_aggressive.setChecked(getattr(settings, 'aggressive_minify', False))
        self.chk_checkpoints.setChecked(getattr(settings, 'save_checkpoints', False))
        self.chk_watch.setChecked(getattr(settings, 'auto_watch', False))

        self._current_template_path = settings.template_path
        self.cmb_format.setCurrentText(settings.output_format)
        self.btn_template.setVisible(settings.output_format == "custom")

        visible_actions = getattr(settings, 'visible_actions', ["preview", "clipboard", "chat", "editor", "file"])
        for act_id, btn in self.action_buttons.items():
            btn.setVisible(act_id in visible_actions)

        visible_checkboxes = getattr(settings, 'visible_checkboxes', ["dedup", "aggressive", "checkpoints", "watch"])
        for cb_id, chk in self.checkbox_buttons.items():
            chk.setVisible(cb_id in visible_checkboxes)

    def retranslate_ui(self):
        self.chk_minify.setText(tr("action_panel.chk_minify"))
        self.chk_comments.setText(tr("action_panel.chk_comments"))
        self.chk_secrets.setText(tr("action_panel.chk_secrets"))
        self.chk_skeleton.setText(tr("action_panel.chk_skeleton"))
        self.chk_dedup.setText(tr("action_panel.chk_dedup"))
        self.chk_aggressive.setText(tr("action_panel.chk_aggressive"))
        self.chk_checkpoints.setText(tr("action_panel.chk_checkpoints"))
        self.chk_watch.setText(tr("action_panel.chk_watch"))
        self.btn_template.setToolTip(tr("action_panel.template.tooltip"))
        self.btn_preview.setText(tr("action_panel.preview.button"))
        self.btn_copy.setText(tr("action_panel.clipboard.button"))
        self.btn_chat.setText(tr("action_panel.chat.button"))
        self.btn_editor.setText(tr("action_panel.editor.button"))
        self.btn_editor.setToolTip(tr("action_panel.editor.tooltip"))
        self.btn_file.setText(tr("action_panel.file.button"))
        if self._plugin_action_buttons(self._plugin_api):
            self.btn_plugins.setText(tr("action_panel.plugins.button"))

    def get_settings(self):
        return {
            'minify': self.chk_minify.isChecked(),
            'remove_comments': self.chk_comments.isChecked(),
            'remove_secrets': self.chk_secrets.isChecked(),
            'skeleton_mode': self.chk_skeleton.isChecked(),
            'deduplicate': self.chk_dedup.isChecked(),
            'aggressive_minify': self.chk_aggressive.isChecked(),
            'save_checkpoints': self.chk_checkpoints.isChecked(),
            'auto_watch': self.chk_watch.isChecked(),
            'output_format': self.cmb_format.currentText(),
            'template_path': getattr(self, '_current_template_path', "")
        }
