from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QTextEdit,
                               QPushButton, QTabWidget, QWidget, QLabel, QLineEdit)

class AdvancedPreviewDialog(QDialog):
    def __init__(self, parent, state, on_close):
        super().__init__(parent)
        self.on_close = on_close
        self.setWindowTitle("Предпросмотр (CodeContext AI)")
        self.resize(1000, 700)

        layout = QVBoxLayout(self)

        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        self.tab_preview = QWidget()
        self.tabs.addTab(self.tab_preview, "📝 Контекст")

        preview_layout = QVBoxLayout(self.tab_preview)
        self.txt_preview = QTextEdit()
        self.txt_preview.setReadOnly(True)
        preview_layout.addWidget(self.txt_preview)

        btn_copy = QPushButton("📋 Копировать всё")
        btn_copy.clicked.connect(self._copy_all)
        preview_layout.addWidget(btn_copy)

    def update_data(self, state):
        self.txt_preview.setPlainText(state.preview_text)

    def _copy_all(self):
        from PySide6.QtGui import QGuiApplication
        cb = QGuiApplication.clipboard()
        cb.setText(self.txt_preview.toPlainText())

    def closeEvent(self, event):
        self.on_close()
        super().closeEvent(event)


class InteractiveTourDialog(QDialog):
    def __init__(self, parent, steps, on_close):
        super().__init__(parent)
        self.steps = steps
        self.on_close = on_close
        self.current_step = 0
        self.setWindowTitle("Интерактивный тур")
        self.resize(700, 500)

        layout = QVBoxLayout(self)

        self.lbl_title = QLabel()
        self.lbl_title.setStyleSheet("font-size: 24px; font-weight: bold;")

        self.txt_desc = QTextEdit()
        self.txt_desc.setReadOnly(True)

        btn_layout = QHBoxLayout()
        self.btn_prev = QPushButton("⬅ Назад")
        self.btn_prev.setProperty("cssClass", "ghost")
        self.btn_prev.clicked.connect(self._prev)

        self.btn_next = QPushButton("Далее ➡")
        self.btn_next.clicked.connect(self._next)

        btn_layout.addWidget(self.btn_prev)
        btn_layout.addStretch()
        btn_layout.addWidget(self.btn_next)

        layout.addWidget(self.lbl_title)
        layout.addWidget(self.txt_desc)
        layout.addLayout(btn_layout)

        self._update_ui()

    def _update_ui(self):
        step = self.steps[self.current_step]
        self.lbl_title.setText(step.get("title", ""))
        self.txt_desc.setPlainText(step.get("text", ""))

        self.btn_prev.setEnabled(self.current_step > 0)
        if self.current_step == len(self.steps) - 1:
            self.btn_next.setText("Начать работу 🚀")
        else:
            self.btn_next.setText("Далее ➡")

    def _prev(self):
        if self.current_step > 0:
            self.current_step -= 1
            self._update_ui()

    def _next(self):
        if self.current_step < len(self.steps) - 1:
            self.current_step += 1
            self._update_ui()
        else:
            self.close()

    def closeEvent(self, event):
        self.on_close()
        super().closeEvent(event)


class EditFolderDialog(QDialog):
    def __init__(self, parent, initial_path):
        super().__init__(parent)
        self.result = None
        self.setWindowTitle("Редактировать путь")
        self.resize(400, 100)

        layout = QVBoxLayout(self)
        self.entry = QLineEdit(initial_path)
        layout.addWidget(self.entry)

        btn_layout = QHBoxLayout()
        btn_ok = QPushButton("OK")
        btn_ok.clicked.connect(self._on_ok)
        btn_layout.addWidget(btn_ok)

        layout.addLayout(btn_layout)

    def _on_ok(self):
        self.result = self.entry.text()
        self.accept()

    def get_input(self):
        return self.result


class JsonPatchDialog(QDialog):
    """Диалог для применения JSON патчей от LLM"""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Применить JSON-патч от ИИ")
        self.resize(650, 450)

        layout = QVBoxLayout(self)

        lbl = QLabel("Вставьте ответ нейросети (массив JSON с инструкциями):")
        lbl.setProperty("cssClass", "muted")
        layout.addWidget(lbl)

        self.txt_patch = QTextEdit()
        self.txt_patch.setPlaceholderText('[\n  {\n    "action": "replace",\n    "file": "main.py",\n    "search": "def test():\\n    pass",\n    "content": "def test():\\n    return True"\n  }\n]')
        layout.addWidget(self.txt_patch)

        btn_layout = QHBoxLayout()
        btn_cancel = QPushButton("Отмена")
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)

        btn_apply = QPushButton("Применить Патч")
        btn_apply.setProperty("cssClass", "success")
        btn_apply.clicked.connect(self.accept)

        btn_layout.addStretch()
        btn_layout.addWidget(btn_cancel)
        btn_layout.addWidget(btn_apply)

        layout.addLayout(btn_layout)

    def get_json(self):
        return self.txt_patch.toPlainText().strip()