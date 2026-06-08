from PySide6.QtWidgets import QWidget, QHBoxLayout, QLabel, QProgressBar
from ..theme_manager import ThemeManager, theme_bus


class StatusBar(QWidget):
    def __init__(self):
        super().__init__()
        self.layout = QHBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)

        self.lbl_status = QLabel("Idle")
        self.lbl_status.setProperty("cssClass", "muted")

        self.progress = QProgressBar()
        self.progress.setTextVisible(False)

        self.lbl_tokens = QLabel("Tokens: 0")
        self.lbl_tokens.setProperty("cssClass", "muted")

        self.layout.addWidget(self.lbl_status)
        self.layout.addWidget(self.progress, 1)
        self.layout.addWidget(self.lbl_tokens)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        s = ThemeManager.get_layout("main_spacing", 12)
        self.layout.setSpacing(s)

    def update_ui(self, msg: str, prog: float, tokens: int, cost: float = 0.0):
        self.lbl_status.setText(msg)
        self.progress.setValue(int(prog * 100))

        cost_str = f" (~${cost:.4f})" if cost > 0 else " (Локальная модель/Free)"
        self.lbl_tokens.setText(f"Tokens: {tokens}{cost_str}")
