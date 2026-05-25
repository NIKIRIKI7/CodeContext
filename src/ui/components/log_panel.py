from PySide6.QtWidgets import QPlainTextEdit
from PySide6.QtCore import Qt

class LogPanel(QPlainTextEdit):
    def __init__(self):
        super().__init__()
        self.setReadOnly(True)
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setProperty("cssClass", "card")

    def update_logs(self, logs):
        self.setPlainText("\n".join(logs))
        self.verticalScrollBar().setValue(self.verticalScrollBar().maximum())