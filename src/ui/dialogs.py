import difflib
from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QTextEdit,
                               QPushButton, QTabWidget, QWidget, QLabel, QLineEdit,
                               QListWidget, QListWidgetItem, QTextBrowser, QSplitter)
from PySide6.QtCore import Qt
from .theme_manager import ThemeManager


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
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Применить JSON-патч от ИИ")
        self.resize(650, 450)

        layout = QVBoxLayout(self)
        lbl = QLabel("Вставьте ответ нейросети (массив JSON с инструкциями):")
        lbl.setProperty("cssClass", "muted")
        layout.addWidget(lbl)

        self.txt_patch = QTextEdit()
        self.txt_patch.setPlaceholderText(
            '[\n  {\n    "action": "replace",\n    "file": "main.py",\n    "search": "def test():\\n    pass",\n    "content": "def test():\\n    return True"\n  }\n]')
        layout.addWidget(self.txt_patch)

        btn_layout = QHBoxLayout()
        btn_cancel = QPushButton("Отмена")
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)

        btn_apply = QPushButton("Далее (Предпросмотр)")
        btn_apply.setProperty("cssClass", "success")
        btn_apply.clicked.connect(self.accept)

        btn_layout.addStretch()
        btn_layout.addWidget(btn_cancel)
        btn_layout.addWidget(btn_apply)
        layout.addLayout(btn_layout)

    def get_json(self):
        return self.txt_patch.toPlainText().strip()


class InteractiveDiffDialog(QDialog):
    """Окно безопасного предпросмотра Diff'ов с возможностью LLM-проверки и принятия исправлений"""

    def __init__(self, parent, prepared_patches, controller):
        super().__init__(parent)
        self.prepared = prepared_patches
        self.controller = controller
        self.setWindowTitle("Предпросмотр изменений (Safety Diff Viewer)")
        self.resize(1200, 800)

        layout = QVBoxLayout(self)
        splitter = QSplitter(Qt.Horizontal)
        layout.addWidget(splitter)

        # --- Левая панель: список файлов ---
        self.list_widget = QListWidget()
        for p in self.prepared:
            item = QListWidgetItem(p['file_target'])
            item.setCheckState(Qt.Checked if p['success'] else Qt.Unchecked)
            if not p['success']:
                item.setToolTip(p['msg'])
            self.list_widget.addItem(item)

        self.list_widget.currentRowChanged.connect(self._on_file_selected)
        splitter.addWidget(self.list_widget)

        # --- Правая панель ---
        right_widget = QWidget()
        right_layout = QVBoxLayout(right_widget)
        right_layout.setContentsMargins(0, 0, 0, 0)

        # 1. Основной браузер Diff
        self.diff_browser = QTextBrowser()
        right_layout.addWidget(self.diff_browser, 3)

        # 2. Блок управления LLM
        llm_container = QWidget()
        llm_container.setProperty("cssClass", "card")
        llm_layout = QVBoxLayout(llm_container)

        llm_top_layout = QHBoxLayout()
        self.btn_llm = QPushButton("🤖 Проверить через LLM")
        self.btn_llm.setProperty("cssClass", "success")
        self.btn_llm.clicked.connect(self._check_llm)

        self.lbl_llm_verdict = QTextEdit()
        self.lbl_llm_verdict.setMaximumHeight(60)
        self.lbl_llm_verdict.setReadOnly(True)
        self.lbl_llm_verdict.setPlaceholderText("Здесь будет вердикт нейросети...")

        llm_top_layout.addWidget(self.btn_llm)
        llm_top_layout.addWidget(self.lbl_llm_verdict, 1)
        llm_layout.addLayout(llm_top_layout)

        # 3. Скрытая панель с предложением кода от LLM
        self.suggestion_panel = QWidget()
        self.suggestion_panel.setVisible(False)
        sug_layout = QVBoxLayout(self.suggestion_panel)
        sug_layout.setContentsMargins(0, 10, 0, 0)

        lbl_sug = QLabel("💡 LLM предлагает исправление (Diff между текущим патчем и вариантом ИИ):")
        lbl_sug.setProperty("cssClass", "heading")
        self.sug_diff_browser = QTextBrowser()
        self.sug_diff_browser.setMaximumHeight(200)

        sug_btn_layout = QHBoxLayout()
        self.btn_accept_sug = QPushButton("✅ Принять предложение LLM")
        self.btn_accept_sug.setProperty("cssClass", "success")
        self.btn_accept_sug.clicked.connect(self._accept_suggestion)

        self.btn_reject_sug = QPushButton("❌ Отклонить")
        self.btn_reject_sug.setProperty("cssClass", "ghost")
        self.btn_reject_sug.clicked.connect(self._reject_suggestion)

        sug_btn_layout.addStretch()
        sug_btn_layout.addWidget(self.btn_reject_sug)
        sug_btn_layout.addWidget(self.btn_accept_sug)

        sug_layout.addWidget(lbl_sug)
        sug_layout.addWidget(self.sug_diff_browser)
        sug_layout.addLayout(sug_btn_layout)

        llm_layout.addWidget(self.suggestion_panel)
        right_layout.addWidget(llm_container, 2)

        splitter.addWidget(right_widget)
        splitter.setSizes([300, 900])

        # --- Подвал с кнопками сохранения ---
        btn_layout = QHBoxLayout()
        btn_cancel = QPushButton("Отмена")
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)

        btn_apply = QPushButton("💾 Сохранить выбранные на диск")
        btn_apply.setProperty("cssClass", "success")
        btn_apply.clicked.connect(self.accept)

        btn_layout.addStretch()
        btn_layout.addWidget(btn_cancel)
        btn_layout.addWidget(btn_apply)
        layout.addLayout(btn_layout)

        self.current_llm_suggestion = None

        # Выбираем первый элемент списка по умолчанию
        if self.prepared:
            self.list_widget.setCurrentRow(0)

    def _generate_html_diff(self, source_text, target_text):
        """Генерирует HTML Diff, строго опираясь на JSON темы."""
        colors = ThemeManager.get_current_colors()
        fonts = ThemeManager.get_font_settings()
        font_family = fonts.get("family", "monospace")

        html_diff = difflib.HtmlDiff(wrapcolumn=90).make_file(
            source_text.splitlines(),
            target_text.splitlines(),
            context=True, numlines=5
        )

        # Динамический CSS из настроек темы
        custom_css = f"""
        <style>
            table.diff {{font-family: {font_family}; width: 100%; border-collapse: collapse; color: {colors.get('text', '#000')};}}
            .diff_header {{background-color: {colors.get('diff_hdr', '#e0e0e0')}; text-align:center;}}
            .diff_next {{background-color: {colors.get('diff_hdr', '#e0e0e0')};}}
            .diff_add {{background-color: {colors.get('diff_add', '#cceeff')};}}
            .diff_chg {{background-color: {colors.get('diff_chg', '#ffffcc')};}}
            .diff_sub {{background-color: {colors.get('diff_sub', '#ffcccc')};}}
            td {{padding: 2px 6px; border: 1px solid {colors.get('border', '#ccc')};}}
        </style>
        """
        html_diff = html_diff.replace('<style type="text/css">', custom_css + '<style type="text/css">')
        return html_diff

    def _on_file_selected(self, idx):
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None
        self.lbl_llm_verdict.clear()

        if idx < 0 or idx >= len(self.prepared): return
        p = self.prepared[idx]

        if not p['success']:
            colors = ThemeManager.get_current_colors()
            err_color = colors.get('danger', 'red')
            self.diff_browser.setHtml(
                f"<h3 style='color:{err_color};'>Ошибка подготовки патча:</h3><p>{p['msg']}</p>"
            )
            return

        html_diff = self._generate_html_diff(p['original_content'], p['patched_content'])
        self.diff_browser.setHtml(html_diff)

    def _check_llm(self):
        idx = self.list_widget.currentRow()
        if idx < 0: return
        patch = self.prepared[idx]

        if not patch['success']: return

        self.btn_llm.setText("⏳ Проверка...")
        self.btn_llm.setEnabled(False)
        self.lbl_llm_verdict.setPlainText("Отправка запроса к LLM...")
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None

        def on_result(result_dict: dict):
            status = result_dict.get('status', 'ERROR')
            reason = result_dict.get('reason', '')
            sug_code = result_dict.get('suggested_code')

            self.lbl_llm_verdict.setPlainText(f"Статус: {status}\nВердикт: {reason}")
            self.btn_llm.setText("🤖 Проверить снова")
            self.btn_llm.setEnabled(True)

            # Если есть предложенный код и он отличается от изначального патча
            if sug_code and sug_code.strip() != patch['patched_content'].strip():
                self.current_llm_suggestion = sug_code
                self.suggestion_panel.setVisible(True)
                sug_html = self._generate_html_diff(patch['patched_content'], sug_code)
                self.sug_diff_browser.setHtml(sug_html)

        # Вызываем контроллер (который пробросит вызов в сервис)
        self.controller.verify_patch_with_llm(patch, on_result)

    def _accept_suggestion(self):
        """Применяет предложенный код от LLM к текущему патчу."""
        idx = self.list_widget.currentRow()
        if idx < 0 or not self.current_llm_suggestion: return

        # Обновляем содержимое патча
        self.prepared[idx]['patched_content'] = self.current_llm_suggestion
        self.prepared[idx]['msg'] += " (✨ Обновлено по совету ИИ)"

        # Скрываем панель
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None
        self.lbl_llm_verdict.setPlainText("✅ Предложение LLM успешно применено к патчу!")

        # Перерисовываем основной Diff с новым (принятым) кодом
        self._on_file_selected(idx)

    def _reject_suggestion(self):
        """Отклоняет предложение LLM."""
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None
        self.lbl_llm_verdict.setPlainText("Предложение LLM отклонено. Используется исходный патч.")

    def get_selected(self):
        selected = []
        for i in range(self.list_widget.count()):
            if self.list_widget.item(i).checkState() == Qt.Checked:
                selected.append(self.prepared[i])
        return selected