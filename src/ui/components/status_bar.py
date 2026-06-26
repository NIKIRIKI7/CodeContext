from PySide6.QtWidgets import QWidget, QHBoxLayout, QLabel, QProgressBar
from ..theme_manager import ThemeManager, theme_bus
from src.i18n import tr


class StatusBar(QWidget):
    def __init__(self):
        super().__init__()
        self.layout = QHBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)

        self.lbl_status = QLabel(tr("status_bar.idle"))
        self.lbl_status.setProperty("cssClass", "muted")

        self.progress = QProgressBar()
        self.progress.setTextVisible(False)

        self.lbl_tokens = QLabel(tr("status_bar.tokens_count", tokens=0))
        self.lbl_tokens.setProperty("cssClass", "muted")

        self.layout.addWidget(self.lbl_status)
        self.layout.addWidget(self.progress, 1)
        self.layout.addWidget(self.lbl_tokens)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def retranslate_ui(self):
        if self.lbl_status.text() == "Idle" or self.lbl_status.text() == tr("status_bar.idle"):
            self.lbl_status.setText(tr("status_bar.idle"))

    def _update_metrics(self):
        s = ThemeManager.get_layout("main_spacing", 12)
        self.layout.setSpacing(s)

    def update_ui(self, msg: str, prog: float, tokens: int, cost: float = 0.0, is_loading: bool = False):
        self.lbl_status.setText(msg)

        if is_loading and prog == 0.0:
            self.progress.setRange(0, 0)
        else:
            self.progress.setRange(0, 100)
            self.progress.setValue(int(prog * 100))

        cost_str = f" (~${cost:.4f})" if cost > 0 else tr("status_bar.local_model")
        self.lbl_tokens.setText(tr("status_bar.tokens_count", tokens=tokens) + cost_str)
