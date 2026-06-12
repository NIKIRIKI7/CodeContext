from src.api import IPlugin, PluginAPI


class HelloWorldPlugin(IPlugin):
    id = "hello_world"
    name = "Hello World"
    version = "1.0.0"

    def on_init(self, api: PluginAPI) -> None:
        api.add_translations("ru", {
            "hello_world.greeting": "Привет, мир! Плагин работает.",
            "hello_world.btn_label": "Сказать «Привет»",
        })

        api.ui.register_sidebar_tab(
            tab_id="hello_world",
            label="Hello",
            widget_factory=lambda: self._build_widget(api),
        )

        api.ui.register_action_button(
            action_id="hello_world_say_hi",
            label="👋 Hello World",
            callback=self._show_hello_dialog,
        )

    def _show_hello_dialog(self):
        from PySide6.QtWidgets import QMessageBox, QApplication
        parent = QApplication.activeWindow()
        QMessageBox.information(parent, "Hello World", "Привет, мир! Плагин работает.")

    def _build_widget(self, api: PluginAPI):
        from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QPushButton
        from src.i18n import tr

        widget = QWidget()
        layout = QVBoxLayout(widget)
        label = QLabel(tr("hello_world.greeting", default="Привет, мир! Плагин работает."))
        btn = QPushButton(tr("hello_world.btn_label", default='Сказать "Привет"'))
        btn.clicked.connect(self._show_hello_dialog)
        layout.addWidget(label)
        layout.addWidget(btn)
        layout.addStretch()
        return widget

    def on_shutdown(self) -> None:
        pass
