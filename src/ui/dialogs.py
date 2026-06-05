import re
from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QTextEdit,
                               QPushButton, QTabWidget, QWidget, QLabel, QLineEdit,
                               QListWidget, QListWidgetItem, QTextBrowser, QSplitter,
                               QPlainTextEdit)
from PySide6.QtCore import Qt
from PySide6.QtGui import QSyntaxHighlighter, QTextCharFormat, QColor, QFont
from .theme_manager import ThemeManager


class ChatDialog(QDialog):
    """Окно прямого общения с LLM, куда уже встроен контекст проекта"""
    def __init__(self, parent, state, controller):
        super().__init__(parent)
        self.state = state
        self.controller = controller
        self.setWindowTitle("AI Chat (CodeContext)")
        self.resize(850, 650)

        layout = QVBoxLayout(self)

        self.chat_history = QTextBrowser()
        self.chat_history.setOpenExternalLinks(True)
        layout.addWidget(self.chat_history, 4)

        input_layout = QHBoxLayout()
        self.input_field = QPlainTextEdit()
        self.input_field.setProperty("cssClass", "chat_input")
        self.input_field.setPlaceholderText("Напишите ваш запрос... (Например: 'Найди баг' или 'Оптимизируй этот файл')")

        self.btn_send = QPushButton("Отправить")
        self.btn_send.setProperty("cssClass", "success")
        self.btn_send.clicked.connect(self._send_message)

        input_layout.addWidget(self.input_field, 4)
        input_layout.addWidget(self.btn_send, 1)
        layout.addLayout(input_layout)

        self.messages = []

    def update_data(self, state):
        if not self.messages:
            self.messages.append({"role": "system", "content": state.chat_context})
            self.chat_history.append("<b>Система:</b> Контекст проекта успешно загружен в память! Можете задавать вопросы.<br><br>")

    def _send_message(self):
        user_text = self.input_field.toPlainText().strip()
        if not user_text:
            return

        self.input_field.clear()
        self.messages.append({"role": "user", "content": user_text})
        self.chat_history.append(f"<b>Вы:</b> {user_text}<br><br>")
        self.btn_send.setEnabled(False)
        self.btn_send.setText("⏳ Ожидание...")

        async def fetch_reply():
            reply = await self.controller._llm_checker.send_chat_message(self.messages, self.controller._store.state.settings)
            from PySide6.QtCore import QTimer
            QTimer.singleShot(0, lambda: self._on_reply(reply))

        from ..utils.async_runtime import AsyncRuntime
        AsyncRuntime.run_coroutine(fetch_reply())

    def _on_reply(self, reply):
        self.messages.append({"role": "assistant", "content": reply})
        self.chat_history.append(f"<b>AI:</b> {reply}<br><br>")
        self.btn_send.setEnabled(True)
        self.btn_send.setText("Отправить")
        self.chat_history.verticalScrollBar().setValue(self.chat_history.verticalScrollBar().maximum())

    def closeEvent(self, event):
        self.controller.close_chat()
        super().closeEvent(event)


class PreviewHighlighter(QSyntaxHighlighter):
    """Простая подсветка синтаксиса для Markdown и XML в окне предпросмотра"""

    def __init__(self, document, theme_colors):
        super().__init__(document)
        self.rules = []

        primary = QColor(theme_colors.get('primary', '#0071e3'))
        comment = QColor(theme_colors.get('text_muted', '#707070'))
        success = QColor(theme_colors.get('success', '#34c759'))

        fmt_heading = QTextCharFormat()
        fmt_heading.setFontWeight(QFont.Bold)
        fmt_heading.setForeground(primary)
        self.rules.append((r'^(#+\s.*|={10,}.*|PROJECT STRUCTURE:|DEPENDENCY GRAPH:)', fmt_heading))

        fmt_tag = QTextCharFormat()
        fmt_tag.setForeground(success)
        self.rules.append((r'<[^>]+>', fmt_tag))

        fmt_fence = QTextCharFormat()
        fmt_fence.setForeground(comment)
        self.rules.append((r'^```.*', fmt_fence))

    def highlightBlock(self, text):
        for pattern, fmt in self.rules:
            for match in re.finditer(pattern, text):
                self.setFormat(match.start(), match.end() - match.start(), fmt)


class AdvancedPreviewDialog(QDialog):
    def __init__(self, parent, state, on_close, controller):
        super().__init__(parent)
        self.on_close = on_close
        self.state = state
        self.controller = controller
        self.setWindowTitle("Предпросмотр (CodeContext AI)")
        self.resize(1100, 700)

        layout = QVBoxLayout(self)
        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        self.tab_preview = QWidget()
        self.tabs.addTab(self.tab_preview, "📝 Контекст")
        preview_layout = QVBoxLayout(self.tab_preview)

        self.splitter = QSplitter(Qt.Horizontal)
        preview_layout.addWidget(self.splitter)

        left_widget = QWidget()
        left_layout = QVBoxLayout(left_widget)
        left_layout.setContentsMargins(0, 0, 0, 0)

        lbl_toc = QLabel("📄 Файлы в промпте:")
        lbl_toc.setProperty("cssClass", "heading")
        left_layout.addWidget(lbl_toc)

        self.list_toc = QListWidget()
        self.list_toc.currentRowChanged.connect(self._on_toc_selected)
        left_layout.addWidget(self.list_toc)
        self.splitter.addWidget(left_widget)

        right_widget = QWidget()
        right_layout = QVBoxLayout(right_widget)
        right_layout.setContentsMargins(0, 0, 0, 0)

        toolbar_layout = QHBoxLayout()
        self.btn_copy_all = QPushButton("📋 Копировать всё")
        self.btn_copy_all.clicked.connect(self._copy_all)

        self.btn_copy_file = QPushButton("✂️ Скопировать только этот файл")
        self.btn_copy_file.setProperty("cssClass", "ghost")
        self.btn_copy_file.setEnabled(False)
        self.btn_copy_file.clicked.connect(self._copy_selected_file)

        toolbar_layout.addWidget(self.btn_copy_all)
        toolbar_layout.addWidget(self.btn_copy_file)
        toolbar_layout.addStretch()
        right_layout.addLayout(toolbar_layout)

        self.txt_preview = QPlainTextEdit()
        self.txt_preview.setReadOnly(True)
        self.txt_preview.setLineWrapMode(QPlainTextEdit.NoWrap)
        right_layout.addWidget(self.txt_preview)

        self.splitter.addWidget(right_widget)
        self.splitter.setSizes([300, 800])

        colors = ThemeManager.get_current_colors()
        self.highlighter = PreviewHighlighter(self.txt_preview.document(), colors)

    def update_data(self, state):
        self.state = state
        self.txt_preview.setPlainText(state.preview_text)

        self.list_toc.clear()
        for f in state.processed_files:
            self.list_toc.addItem(f.path)

    def _on_toc_selected(self, idx):
        if idx < 0 or idx >= len(self.state.processed_files):
            self.btn_copy_file.setEnabled(False)
            return

        self.btn_copy_file.setEnabled(True)
        selected_file = self.state.processed_files[idx]

        # Делегируем логику определения маркеров контроллеру
        search_strs = self.controller.get_search_markers_for_preview(selected_file.path)

        cursor = self.txt_preview.textCursor()
        cursor.setPosition(0)
        self.txt_preview.setTextCursor(cursor)

        for s in search_strs:
            if self.txt_preview.find(s):
                self.txt_preview.centerCursor()
                break

    def _copy_all(self):
        # Делегируем буфер обмена контроллеру
        self.controller.copy_to_clipboard(self.txt_preview.toPlainText())

    def _copy_selected_file(self):
        idx = self.list_toc.currentRow()
        if idx < 0 or idx >= len(self.state.processed_files):
            return
        selected_file = self.state.processed_files[idx]
        # Делегируем буфер обмена контроллеру
        self.controller.copy_to_clipboard(selected_file.content)

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
        self.lbl_title.setProperty("cssClass", "tour_title")
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

        self.list_widget = QListWidget()
        for p in self.prepared:
            item = QListWidgetItem(p['file_target'])
            item.setCheckState(Qt.Checked if p['success'] else Qt.Unchecked)
            if not p['success']:
                item.setToolTip(p['msg'])
            self.list_widget.addItem(item)
        self.list_widget.currentRowChanged.connect(self._on_file_selected)
        splitter.addWidget(self.list_widget)

        right_widget = QWidget()
        right_layout = QVBoxLayout(right_widget)
        right_layout.setContentsMargins(0, 0, 0, 0)
        self.diff_browser = QTextBrowser()
        right_layout.addWidget(self.diff_browser, 3)

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
        if self.prepared:
            self.list_widget.setCurrentRow(0)

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

        # Делегируем генерацию HTML Diff контроллеру
        colors = ThemeManager.get_current_colors()
        fonts = ThemeManager.get_font_settings()
        html_diff = self.controller.generate_html_diff(p['original_content'], p['patched_content'], colors, fonts)
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
            if sug_code and sug_code.strip() != patch['patched_content'].strip():
                self.current_llm_suggestion = sug_code
                self.suggestion_panel.setVisible(True)

                # Делегируем генерацию HTML Diff контроллеру
                colors = ThemeManager.get_current_colors()
                fonts = ThemeManager.get_font_settings()
                sug_html = self.controller.generate_html_diff(patch['patched_content'], sug_code, colors, fonts)
                self.sug_diff_browser.setHtml(sug_html)

        self.controller.verify_patch_with_llm(patch, on_result)

    def _accept_suggestion(self):
        """Применяет предложенный код от LLM к текущему патчу."""
        idx = self.list_widget.currentRow()
        if idx < 0 or not self.current_llm_suggestion: return
        self.prepared[idx]['patched_content'] = self.current_llm_suggestion
        self.prepared[idx]['msg'] += " (✨ Обновлено по совету ИИ)"
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None
        self.lbl_llm_verdict.setPlainText("✅ Предложение LLM успешно применено к патчу!")
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


class UpdateDialog(QDialog):
    """Диалог обновления с полной поддержкой ThemeManager и Redux State."""

    def __init__(self, parent, update_info, on_close, controller):
        super().__init__(parent)
        self.on_close = on_close
        self.controller = controller

        self.setWindowTitle("Обновления CodeContext AI")
        self.resize(550, 450)
        self.layout = QVBoxLayout(self)

        self.lbl_title = QLabel()
        self.lbl_title.setProperty("cssClass", "heading")
        self.layout.addWidget(self.lbl_title)

        self.notes = QTextBrowser()
        self.notes.setOpenExternalLinks(True)
        self.layout.addWidget(self.notes)

        self.btn_layout = QHBoxLayout()
        self.btn_cancel = QPushButton()
        self.btn_cancel.setProperty("cssClass", "ghost")
        self.btn_cancel.clicked.connect(self.reject)

        self.btn_update = QPushButton()
        self.btn_update.setProperty("cssClass", "success")
        self.btn_update.clicked.connect(self._do_update)

        self.btn_layout.addStretch()
        self.btn_layout.addWidget(self.btn_cancel)
        self.btn_layout.addWidget(self.btn_update)
        self.layout.addLayout(self.btn_layout)

        self.update_data(update_info)

    def update_data(self, update_info):
        self.update_info = update_info
        status = update_info.get('status', 'checking')
        version = update_info.get('version', '')
        notes_text = update_info.get('notes', '')

        if status == 'checking':
            self.lbl_title.setText(f"⏳ Поиск обновлений (v{version})...")
            self.notes.setMarkdown(notes_text)
            self.btn_update.hide()
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText("Отмена")

        elif status == 'available':
            self.lbl_title.setText(f"🎉 Доступна новая версия: {version}")
            self.notes.setMarkdown(notes_text)
            self.btn_update.show()
            self.btn_update.setText("Скачать и обновить")
            self.btn_update.setEnabled(True)
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText("Позже")

        elif status == 'latest':
            self.lbl_title.setText("✅ У вас самая актуальная версия")
            self.notes.setMarkdown(notes_text)
            self.btn_update.hide()
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText("Закрыть")

        else:
            self.lbl_title.setText("❌ Ошибка при проверке/установке")
            self.notes.setMarkdown(notes_text)
            self.btn_update.hide()
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText("Закрыть")

        self.lbl_title.style().unpolish(self.lbl_title)
        self.lbl_title.style().polish(self.lbl_title)

    def _do_update(self):
        self.btn_update.setEnabled(False)
        self.btn_update.setText("Загрузка...")
        self.btn_cancel.setEnabled(False)
        self.lbl_title.setText("⬇️ Идёт скачивание и установка...")
        self.notes.setMarkdown(
            "Пожалуйста, не закрывайте приложение.\n\n"
            "После загрузки программа будет автоматически перезапущена.\n"
            "Вы можете наблюдать за прогрессом загрузки в статус-баре на главном экране."
        )
        self.controller.apply_update(self.update_info['download_url'])

    def closeEvent(self, event):
        self.on_close()
        super().closeEvent(event)