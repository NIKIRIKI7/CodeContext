import re
from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QTextEdit,
                               QPushButton, QTabWidget, QWidget, QLabel, QLineEdit,
                               QListWidget, QListWidgetItem, QTextBrowser, QSplitter,
                               QPlainTextEdit, QComboBox, QCheckBox, QMessageBox)
from PySide6.QtCore import Qt
from PySide6.QtGui import QSyntaxHighlighter, QTextCharFormat, QColor, QFont
from .theme_manager import ThemeManager
from src.i18n import tr


class ChatDialog(QDialog):
    """Окно прямого общения с LLM, куда уже встроен контекст проекта"""
    def __init__(self, parent, state, controller):
        super().__init__(parent)
        self.state = state
        self.controller = controller
        self.setWindowTitle(tr("dialogs.ai_chat_title"))
        self.resize(850, 650)

        layout = QVBoxLayout(self)

        self.chat_history = QTextBrowser()
        self.chat_history.setOpenExternalLinks(True)
        layout.addWidget(self.chat_history, 4)

        input_layout = QHBoxLayout()
        self.input_field = QPlainTextEdit()
        self.input_field.setProperty("cssClass", "chat_input")
        self.input_field.setPlaceholderText(tr("dialogs.chat.placeholder"))

        self.btn_send = QPushButton(tr("dialogs.chat.send"))
        self.btn_send.setProperty("cssClass", "success")
        self.btn_send.clicked.connect(self._send_message)

        input_layout.addWidget(self.input_field, 4)
        input_layout.addWidget(self.btn_send, 1)
        layout.addLayout(input_layout)

        self.messages = []

    def update_data(self, state):
        if not self.messages:
            self.messages.append({"role": "system", "content": state.chat_context})
            self.chat_history.append(tr("dialogs.chat.system_loaded"))

    def _send_message(self):
        user_text = self.input_field.toPlainText().strip()
        if not user_text:
            return

        self.input_field.clear()
        self.messages.append({"role": "user", "content": user_text})
        self.chat_history.append(f"<b>{tr('dialogs.chat.you_label')}:</b> {user_text}<br><br>")
        self.btn_send.setEnabled(False)
        self.btn_send.setText(tr("dialogs.chat.waiting"))

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
        self.btn_send.setText(tr("dialogs.chat.send"))
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
        self._diff_cache = {}

        self.setWindowTitle(tr("dialogs.preview.title"))
        self.resize(1100, 700)

        layout = QVBoxLayout(self)
        self.tabs = QTabWidget()
        layout.addWidget(self.tabs)

        self.tab_preview = QWidget()
        self.tabs.addTab(self.tab_preview, tr("dialogs.preview.tab_prompt"))
        preview_layout = QVBoxLayout(self.tab_preview)

        self.splitter = QSplitter(Qt.Horizontal)
        preview_layout.addWidget(self.splitter)

        left_widget = QWidget()
        left_layout = QVBoxLayout(left_widget)
        left_layout.setContentsMargins(0, 0, 0, 0)

        lbl_toc = QLabel(tr("dialogs.preview.files_in_prompt"))
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
        self.btn_copy_all = QPushButton(tr("dialogs.preview.copy_all"))
        self.btn_copy_all.clicked.connect(self._copy_all)
        self.btn_copy_file = QPushButton(tr("dialogs.preview.copy_file"))
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

        self.tab_diff = QWidget()
        self.tabs.addTab(self.tab_diff, tr("dialogs.preview.tab_diff"))
        diff_layout = QVBoxLayout(self.tab_diff)

        diff_top_layout = QHBoxLayout()
        diff_top_layout.addWidget(QLabel(tr("dialogs.preview.select_file")))
        self.cmb_diff_files = QComboBox()
        self.cmb_diff_files.setSizeAdjustPolicy(QComboBox.AdjustToContents)
        self.cmb_diff_files.currentTextChanged.connect(self._on_diff_file_changed)
        diff_top_layout.addWidget(self.cmb_diff_files, 1)

        self.lbl_savings = QLabel(tr("dialogs.preview.savings_initial"))
        self.lbl_savings.setProperty("cssClass", "heading")
        self.lbl_savings.setStyleSheet("color: #16a34a;")
        diff_top_layout.addWidget(self.lbl_savings)

        diff_layout.addLayout(diff_top_layout)
        self.diff_browser = QTextBrowser()
        self.diff_browser.setOpenExternalLinks(False)
        diff_layout.addWidget(self.diff_browser)

    def update_data(self, state):
        self.state = state
        self.list_toc.clear()
        self.cmb_diff_files.blockSignals(True)
        self.cmb_diff_files.clear()

        for f in state.processed_files:
            self.list_toc.addItem(f.path)

        for d in state.before_after_data:
            self.cmb_diff_files.addItem(d["path"])

        self.cmb_diff_files.blockSignals(False)

        if state.processed_files:
            self.list_toc.setCurrentRow(0)

        if state.before_after_data:
            self.cmb_diff_files.setCurrentIndex(0)
            self._on_diff_file_changed(state.before_after_data[0]["path"])

    def _on_toc_selected(self, idx):
        if idx < 0 or idx >= len(self.state.processed_files):
            self.btn_copy_file.setEnabled(False)
            return
        self.btn_copy_file.setEnabled(True)
        selected_file = self.state.processed_files[idx]

        self.txt_preview.setPlainText(f"FILE: {selected_file.path}\n\n{selected_file.content}")
        self.txt_preview.verticalScrollBar().setValue(0)

        if hasattr(self, 'highlighter') and self.highlighter:
            self.highlighter.rehighlight()

    def _on_diff_file_changed(self, path):
        if not path:
            return
        data = next((d for d in self.state.before_after_data if d["path"] == path), None)
        if not data:
            return

        original = data["original"]
        processed = data["processed"]

        t_orig = len(original) // 4
        t_proc = len(processed) // 4
        saved = t_orig - t_proc
        percent = (saved / t_orig * 100) if t_orig > 0 else 0

        self.lbl_savings.setText(tr("dialogs.preview.savings", original=t_orig, processed=t_proc, percent=percent))

        if path in self._diff_cache:
            self.diff_browser.setHtml(self._diff_cache[path])
            return

        colors = ThemeManager.get_current_colors()
        fonts = ThemeManager.get_font_settings()
        self.diff_browser.setHtml("<div style='padding:20px;'>Generating diff...</div>")

        async def _generate_diff_bg():
            import asyncio
            html_diff = await asyncio.to_thread(
                self.controller.generate_html_diff, original, processed, colors, fonts
            )
            self._diff_cache[path] = html_diff
            from PySide6.QtCore import QTimer
            QTimer.singleShot(0, lambda: self.diff_browser.setHtml(html_diff))

        from src.utils.async_runtime import AsyncRuntime
        AsyncRuntime.run_coroutine(_generate_diff_bg())

    def _copy_all(self):
        self.controller.copy_to_clipboard(self.txt_preview.toPlainText())

    def _copy_selected_file(self):
        idx = self.list_toc.currentRow()
        if idx < 0 or idx >= len(self.state.processed_files):
            return
        selected_file = self.state.processed_files[idx]
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
        self.setWindowTitle(tr("dialogs.tour.title"))
        self.resize(700, 500)
        layout = QVBoxLayout(self)
        self.lbl_title = QLabel()
        self.lbl_title.setProperty("cssClass", "tour_title")
        self.txt_desc = QTextEdit()
        self.txt_desc.setReadOnly(True)
        btn_layout = QHBoxLayout()
        self.btn_prev = QPushButton(tr("dialogs.tour.prev"))
        self.btn_prev.setProperty("cssClass", "ghost")
        self.btn_prev.clicked.connect(self._prev)
        self.btn_next = QPushButton(tr("dialogs.tour.next"))
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
            self.btn_next.setText(tr("dialogs.tour.finish"))
        else:
            self.btn_next.setText(tr("dialogs.tour.next"))

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
        self.setWindowTitle(tr("dialogs.edit_folder.title"))
        self.resize(400, 100)
        layout = QVBoxLayout(self)
        self.entry = QLineEdit(initial_path)
        layout.addWidget(self.entry)
        btn_layout = QHBoxLayout()
        btn_ok = QPushButton(tr("dialogs.edit_folder.ok"))
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
        self.setWindowTitle(tr("dialogs.json_patch.title"))
        self.resize(650, 450)
        layout = QVBoxLayout(self)
        lbl = QLabel(tr("dialogs.json_patch.instruction"))
        lbl.setProperty("cssClass", "muted")
        layout.addWidget(lbl)
        self.txt_patch = QTextEdit()
        self.txt_patch.setPlaceholderText(
            '[\n  {\n    "action": "replace",\n    "file": "main.py",\n    "search": "def test():\\n    pass",\n    "content": "def test():\\n    return True"\n  }\n]')
        layout.addWidget(self.txt_patch)
        btn_layout = QHBoxLayout()
        btn_cancel = QPushButton(tr("dialogs.json_patch.cancel"))
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)
        btn_apply = QPushButton(tr("dialogs.json_patch.apply"))
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
        self.setWindowTitle(tr("dialogs.diff.title"))
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
        self.btn_llm = QPushButton(tr("dialogs.diff.check_llm"))
        self.btn_llm.setProperty("cssClass", "success")
        self.btn_llm.clicked.connect(self._check_llm)
        self.lbl_llm_verdict = QTextEdit()
        self.lbl_llm_verdict.setMaximumHeight(60)
        self.lbl_llm_verdict.setReadOnly(True)
        self.lbl_llm_verdict.setPlaceholderText(tr("dialogs.diff.verdict_placeholder"))
        llm_top_layout.addWidget(self.btn_llm)
        llm_top_layout.addWidget(self.lbl_llm_verdict, 1)
        llm_layout.addLayout(llm_top_layout)

        self.suggestion_panel = QWidget()
        self.suggestion_panel.setVisible(False)
        sug_layout = QVBoxLayout(self.suggestion_panel)
        sug_layout.setContentsMargins(0, 10, 0, 0)
        lbl_sug = QLabel(tr("dialogs.diff.suggestion_label"))
        lbl_sug.setProperty("cssClass", "heading")
        self.sug_diff_browser = QTextBrowser()
        self.sug_diff_browser.setMaximumHeight(200)

        sug_btn_layout = QHBoxLayout()
        self.btn_accept_sug = QPushButton(tr("dialogs.diff.accept_suggestion"))
        self.btn_accept_sug.setProperty("cssClass", "success")
        self.btn_accept_sug.clicked.connect(self._accept_suggestion)
        self.btn_reject_sug = QPushButton(tr("dialogs.diff.reject_suggestion"))
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
        btn_cancel = QPushButton(tr("dialogs.diff.cancel"))
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)
        btn_apply = QPushButton(tr("dialogs.diff.save_selected"))
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
                f"<h3 style='color:{err_color};'>{tr('dialogs.diff.patch_error')}</h3><p>{p['msg']}</p>"
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
        self.btn_llm.setText(tr("dialogs.diff.checking"))
        self.btn_llm.setEnabled(False)
        self.lbl_llm_verdict.setPlainText(tr("dialogs.diff.sending_request"))
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None

        def on_result(result_dict: dict):
            status = result_dict.get('status', 'ERROR')
            reason = result_dict.get('reason', '')
            sug_code = result_dict.get('suggested_code')
            self.lbl_llm_verdict.setPlainText(tr("dialogs.diff.verdict", status=status, reason=reason))
            self.btn_llm.setText(tr("dialogs.diff.check_again"))
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
        self.prepared[idx]['msg'] += tr("dialogs.diff.updated_by_ai")
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None
        self.lbl_llm_verdict.setPlainText(tr("dialogs.diff.suggestion_applied"))
        self._on_file_selected(idx)

    def _reject_suggestion(self):
        """Отклоняет предложение LLM."""
        self.suggestion_panel.setVisible(False)
        self.current_llm_suggestion = None
        self.lbl_llm_verdict.setPlainText(tr("dialogs.diff.suggestion_rejected"))

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

        self.setWindowTitle(tr("dialogs.update.title"))
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
            self.lbl_title.setText(tr("dialogs.update.checking", version=version))
            self.notes.setMarkdown(notes_text)
            self.btn_update.hide()
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText(tr("dialogs.update.cancel"))

        elif status == 'available':
            self.lbl_title.setText(tr("dialogs.update.available", version=version))
            self.notes.setMarkdown(notes_text)
            self.btn_update.show()
            self.btn_update.setText(tr("dialogs.update.download"))
            self.btn_update.setEnabled(True)
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText(tr("dialogs.update.later"))

        elif status == 'latest':
            self.lbl_title.setText(tr("dialogs.update.latest"))
            self.notes.setMarkdown(notes_text)
            self.btn_update.hide()
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText(tr("dialogs.update.close"))

        else:
            self.lbl_title.setText(tr("dialogs.update.error"))
            self.notes.setMarkdown(notes_text)
            self.btn_update.hide()
            self.btn_cancel.setEnabled(True)
            self.btn_cancel.setText(tr("dialogs.update.close"))

        self.lbl_title.style().unpolish(self.lbl_title)
        self.lbl_title.style().polish(self.lbl_title)

    def _do_update(self):
        self.btn_update.setEnabled(False)
        self.btn_update.setText(tr("dialogs.update.downloading"))
        self.btn_cancel.setEnabled(False)
        self.lbl_title.setText(tr("dialogs.update.installing"))
        self.notes.setMarkdown(tr("dialogs.update.install_notes"))
        self.controller.apply_update(self.update_info['download_url'])

    def closeEvent(self, event):
        self.on_close()
        super().closeEvent(event)


class CommandPaletteDialog(QDialog):
    def __init__(self, parent, commands, on_close):
        super().__init__(parent)
        self.commands = commands
        self.on_close = on_close

        self.setWindowFlags(Qt.Popup | Qt.FramelessWindowHint | Qt.NoDropShadowWindowHint)
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.resize(600, 400)

        self.container = QWidget(self)
        self.container.setProperty("cssClass", "card")
        self.container.resize(self.width(), self.height())

        layout = QVBoxLayout(self.container)
        layout.setContentsMargins(10, 10, 10, 10)
        layout.setSpacing(5)

        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText(tr("dialogs.command_palette.placeholder"))
        self.search_input.textChanged.connect(self._filter_commands)
        self.search_input.returnPressed.connect(self._execute_selected)

        self.list_widget = QListWidget()
        self.list_widget.itemClicked.connect(self._execute_selected)

        layout.addWidget(self.search_input)
        layout.addWidget(self.list_widget)

        self._populate_list()

    def _populate_list(self):
        self.list_widget.clear()
        for cmd_name in self.commands.keys():
            item = QListWidgetItem(f"⚡ {cmd_name}")
            item.setData(Qt.UserRole, cmd_name)
            self.list_widget.addItem(item)
        if self.list_widget.count() > 0:
            self.list_widget.setCurrentRow(0)

    def _filter_commands(self, text):
        query = text.lower()
        has_visible = False
        for i in range(self.list_widget.count()):
            item = self.list_widget.item(i)
            match = query in item.text().lower()
            item.setHidden(not match)
            if match and not has_visible:
                self.list_widget.setCurrentItem(item)
                has_visible = True

    def _execute_selected(self):
        item = self.list_widget.currentItem()
        if item and not item.isHidden():
            cmd_name = item.data(Qt.UserRole)
            action = self.commands.get(cmd_name)
            if action:
                self.close()
                action()

    def keyPressEvent(self, event):
        if event.key() == Qt.Key_Down:
            row = self.list_widget.currentRow()
            while row < self.list_widget.count() - 1:
                row += 1
                if not self.list_widget.item(row).isHidden():
                    self.list_widget.setCurrentRow(row)
                    break
            return
        elif event.key() == Qt.Key_Up:
            row = self.list_widget.currentRow()
            while row > 0:
                row -= 1
                if not self.list_widget.item(row).isHidden():
                    self.list_widget.setCurrentRow(row)
                    break
            return
        elif event.key() == Qt.Key_Escape:
            self.close()
            return
        super().keyPressEvent(event)

    def closeEvent(self, event):
        self.on_close()
        super().closeEvent(event)


class UICustomizationDialog(QDialog):
    def __init__(self, parent, settings, on_save):
        super().__init__(parent)
        self.on_save = on_save
        self.setWindowTitle(tr("dialogs.ui_customization.title"))
        self.resize(420, 480)

        layout = QVBoxLayout(self)

        lbl_tabs = QLabel(tr("dialogs.ui_customization.visible_tabs"))
        lbl_tabs.setProperty("cssClass", "heading")
        layout.addWidget(lbl_tabs)

        self.tab_checks = {}
        tab_defs = [
            ("sources", tr("dialogs.ui_customization.tab_sources")),
            ("filters", tr("dialogs.ui_customization.tab_filters")),
            ("prompts", tr("dialogs.ui_customization.tab_prompts")),
            ("llm_os", tr("dialogs.ui_customization.tab_llm_os")),
            ("appearance", tr("dialogs.ui_customization.tab_appearance")),
        ]
        for tab_id, tab_label in tab_defs:
            chk = QCheckBox(tab_label)
            chk.setChecked(tab_id in settings.visible_tabs)
            self.tab_checks[tab_id] = chk
            layout.addWidget(chk)

        layout.addSpacing(10)
        lbl_actions = QLabel(tr("dialogs.ui_customization.visible_actions"))
        lbl_actions.setProperty("cssClass", "heading")
        layout.addWidget(lbl_actions)

        self.action_checks = {}
        action_defs = [
            ("preview", tr("dialogs.ui_customization.action_preview")),
            ("clipboard", tr("dialogs.ui_customization.action_clipboard")),
            ("chat", tr("dialogs.ui_customization.action_chat")),
            ("editor", tr("dialogs.ui_customization.action_editor")),
            ("file", tr("dialogs.ui_customization.action_file")),
        ]
        for act_id, act_label in action_defs:
            chk = QCheckBox(act_label)
            chk.setChecked(act_id in settings.visible_actions)
            self.action_checks[act_id] = chk
            layout.addWidget(chk)

        layout.addSpacing(10)
        lbl_checkboxes = QLabel(tr("dialogs.ui_customization.visible_checkboxes"))
        lbl_checkboxes.setProperty("cssClass", "heading")
        layout.addWidget(lbl_checkboxes)

        self.checkbox_checks = {}
        checkbox_defs = [
            ("dedup", tr("dialogs.ui_customization.checkbox_dedup")),
            ("aggressive", tr("dialogs.ui_customization.checkbox_aggressive")),
            ("checkpoints", tr("dialogs.ui_customization.checkbox_checkpoints")),
            ("watch", tr("dialogs.ui_customization.checkbox_watch")),
        ]
        for cb_id, cb_label in checkbox_defs:
            chk = QCheckBox(cb_label)
            chk.setChecked(cb_id in settings.visible_checkboxes)
            self.checkbox_checks[cb_id] = chk
            layout.addWidget(chk)

        layout.addStretch()

        btn_layout = QHBoxLayout()
        btn_cancel = QPushButton(tr("dialogs.ui_customization.cancel"))
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)
        btn_save = QPushButton(tr("dialogs.ui_customization.save"))
        btn_save.setProperty("cssClass", "success")
        btn_save.clicked.connect(self._save)
        btn_layout.addStretch()
        btn_layout.addWidget(btn_cancel)
        btn_layout.addWidget(btn_save)
        layout.addLayout(btn_layout)

    def _save(self):
        visible_tabs = [tid for tid, chk in self.tab_checks.items() if chk.isChecked()]
        visible_actions = [aid for aid, chk in self.action_checks.items() if chk.isChecked()]
        visible_checkboxes = [cid for cid, chk in self.checkbox_checks.items() if chk.isChecked()]
        self.on_save(visible_tabs, visible_actions, visible_checkboxes)
        self.accept()


class BugReportDialog(QDialog):
    def __init__(self, parent, controller):
        super().__init__(parent)
        self.controller = controller
        self.setWindowTitle(tr("dialogs.bug_report.title"))
        self.resize(600, 450)

        layout = QVBoxLayout(self)

        lbl_desc = QLabel(tr("dialogs.bug_report.description"))
        lbl_desc.setProperty("cssClass", "heading")
        layout.addWidget(lbl_desc)

        self.txt_desc = QTextEdit()
        self.txt_desc.setPlaceholderText(tr("dialogs.bug_report.placeholder"))
        layout.addWidget(self.txt_desc)

        lbl_logs = QLabel(tr("dialogs.bug_report.logs_label"))
        lbl_logs.setProperty("cssClass", "heading")
        layout.addWidget(lbl_logs)

        self.cmb_logs = QComboBox()
        self.cmb_logs.addItems([
            tr("dialogs.bug_report.logs_today"),
            tr("dialogs.bug_report.logs_all"),
            tr("dialogs.bug_report.logs_none")
        ])
        layout.addWidget(self.cmb_logs)

        btn_layout = QHBoxLayout()
        btn_cancel = QPushButton(tr("dialogs.diff.cancel"))
        btn_cancel.setProperty("cssClass", "ghost")
        btn_cancel.clicked.connect(self.reject)

        btn_send = QPushButton(tr("dialogs.bug_report.send"))
        btn_send.setProperty("cssClass", "success")
        btn_send.clicked.connect(self._prepare_and_send)

        btn_layout.addStretch()
        btn_layout.addWidget(btn_cancel)
        btn_layout.addWidget(btn_send)
        layout.addLayout(btn_layout)

    def _get_logs(self, mode_idx):
        if mode_idx == 2:
            return ""

        from src.utils.config import get_app_data_dir
        import os, datetime, re

        log_file = os.path.join(get_app_data_dir(), "logs", "app.log")
        if not os.path.exists(log_file):
            return "Log file not found."

        try:
            with open(log_file, "r", encoding="utf-8", errors="replace") as f:
                lines = f.readlines()
        except Exception as e:
            return f"Error reading logs: {e}"

        if mode_idx == 1:
            return "".join(lines[-500:])

        today_str = datetime.datetime.now().strftime("%Y-%m-%d")
        filtered = []
        for line in lines:
            if re.match(r'^\d{4}-\d{2}-\d{2}', line):
                if line.startswith(today_str):
                    filtered.append(line)
            else:
                if filtered and not line.startswith(today_str):
                    filtered.append(line)

        return "".join(filtered[-500:])

    def _prepare_and_send(self):
        import pyperclip
        import platform as pf
        from src.utils.config import get_app_version
        from PySide6.QtGui import QDesktopServices
        from PySide6.QtCore import QUrl

        desc = self.txt_desc.toPlainText().strip()
        if not desc:
            QMessageBox.warning(self, tr("sidebar.error.title"), tr("dialogs.bug_report.empty_desc"))
            return

        logs = self._get_logs(self.cmb_logs.currentIndex())
        sys_info = f"OS: {pf.system()} {pf.release()}\nVersion: {get_app_version()}"

        issue_body = (
            f"### Description\n{desc}\n\n"
            f"### Environment\n```text\n{sys_info}\n```\n"
        )
        if logs:
            issue_body += f"\n### Logs\n<details><summary>Click to expand</summary>\n\n```text\n{logs}\n```\n</details>\n"

        try:
            pyperclip.copy(issue_body)
        except Exception as e:
            QMessageBox.warning(self, tr("sidebar.error.title"), tr("dialogs.clipboard_error", error=e))
            return

        QMessageBox.information(
            self,
            tr("dialogs.bug_report.copied_title"),
            tr("dialogs.bug_report.copied_msg")
        )

        url = "https://github.com/NIKIRIKI7/CodeContext/issues/new"
        QDesktopServices.openUrl(QUrl(url))
        self.accept()