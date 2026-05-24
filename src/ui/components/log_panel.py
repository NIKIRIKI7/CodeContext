from PySide6.QtWidgets import QPlainTextEdit

class LogPanel(QPlainTextEdit):
    def __init__(self):
        super().__init__()
        self.setReadOnly(True)

    def update_logs(self, logs):
        self.setPlainText("\n".join(logs))
        self.verticalScrollBar().setValue(self.verticalScrollBar().maximum())