import customtkinter as ctk
from tkinter import filedialog
from ..utils.logger import app_logger
from .theme import AppleTheme


class EditFolderDialog(ctk.CTkToplevel):
    def __init__(self, parent, initial_path: str):
        super().__init__(parent)
        self.title("Редактирование")
        self.geometry(AppleTheme.WIN_EDIT)
        self.resizable(False, False)
        self.result = None
        self.transient(parent)
        self.grab_set()

        self.configure(fg_color=AppleTheme.CANVAS)

        ctk.CTkLabel(self, text="Измените путь:", font=AppleTheme.FONT_BODY, text_color=AppleTheme.INK).pack(
            pady=(AppleTheme.SP_24, AppleTheme.SP_8))

        self.input_frame = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
        self.input_frame.pack(fill="x", padx=AppleTheme.SP_24, pady=AppleTheme.SP_12)

        self.entry = ctk.CTkEntry(self.input_frame, font=AppleTheme.FONT_BODY, corner_radius=AppleTheme.RADIUS_SMALL)
        self.entry.pack(side="left", fill="x", expand=True, padx=(AppleTheme.SP_0, AppleTheme.SP_12))
        self.entry.insert(0, initial_path)

        self.btn_browse = ctk.CTkButton(
            self.input_frame, text="📁", width=AppleTheme.SP_40,
            fg_color=AppleTheme.FOG, text_color=AppleTheme.INK, hover_color=AppleTheme.BORDER,
            corner_radius=AppleTheme.RADIUS_SMALL, command=self._on_browse
        )
        self.btn_browse.pack(side="right")

        self.btn_frame = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
        self.btn_frame.pack(fill="x", padx=AppleTheme.SP_24, pady=AppleTheme.SP_8)

        self.btn_ok = ctk.CTkButton(
            self.btn_frame, text="OK", width=100,
            fg_color=AppleTheme.AZURE, text_color="#ffffff", hover_color=AppleTheme.AZURE_HOVER,
            corner_radius=AppleTheme.RADIUS_PILL, font=AppleTheme.FONT_BUTTON, command=self._on_ok
        )
        self.btn_ok.pack(side="left", expand=True)

        self.btn_cancel = ctk.CTkButton(
            self.btn_frame, text="Отмена", width=100,
            fg_color=AppleTheme.TRANSPARENT, hover_color=AppleTheme.FOG, text_color=AppleTheme.INK,
            corner_radius=AppleTheme.RADIUS_PILL, font=AppleTheme.FONT_BODY, command=self.destroy
        )
        self.btn_cancel.pack(side="right", expand=True)

        self.protocol("WM_DELETE_WINDOW", self.destroy)
        self.entry.focus_set()
        self.wait_window()

    def _on_browse(self):
        path = filedialog.askdirectory()
        if path:
            self.entry.delete(0, 'end')
            self.entry.insert(0, path.replace('/', '\\'))

    def _on_ok(self):
        self.result = self.entry.get()
        self.destroy()

    def get_input(self):
        return self.result


class AdvancedPreviewDialog(ctk.CTkToplevel):
    def __init__(self, parent, state, on_close_callback):
        super().__init__(parent)
        self.title("Deep Preview")
        self.geometry(AppleTheme.WIN_PREVIEW)
        self.transient(parent)
        self.configure(fg_color=AppleTheme.CANVAS)
        self.on_close_callback = on_close_callback

        self.app_state = state

        self.tabview = ctk.CTkTabview(
            self,
            fg_color=AppleTheme.CARD,
            segmented_button_selected_color=AppleTheme.FOG,
            segmented_button_selected_hover_color=AppleTheme.BORDER,
            segmented_button_unselected_color=AppleTheme.CANVAS,  # ИСПРАВЛЕНО: Вместо TRANSPARENT
            segmented_button_unselected_hover_color=AppleTheme.FOG,
            text_color=AppleTheme.INK,
            corner_radius=AppleTheme.RADIUS_CARD
        )
        self.tabview.pack(fill="both", expand=True, padx=AppleTheme.SP_20, pady=AppleTheme.SP_20)

        self.tab_preview = self.tabview.add("📝 Контекст")
        self.tab_diff = self.tabview.add("⚖️ До/После")
        self.tab_history = self.tabview.add("🕒 История")

        self._build_preview()
        self._build_diff()
        self._build_history()

        self.protocol("WM_DELETE_WINDOW", self._close)

    def _build_preview(self):
        self.txt_preview = ctk.CTkTextbox(
            self.tab_preview, wrap="word",
            font=AppleTheme.FONT_CODE, corner_radius=AppleTheme.RADIUS_SMALL,
            fg_color=AppleTheme.CANVAS, text_color=AppleTheme.INK
        )
        self.txt_preview.pack(fill="both", expand=True, pady=(AppleTheme.SP_0, AppleTheme.SP_16))
        self.txt_preview.insert("1.0", self.app_state.preview_text)
        self.txt_preview.configure(state="disabled")

        btn_copy = ctk.CTkButton(
            self.tab_preview, text="📋 Копировать всё",
            fg_color=AppleTheme.AZURE, text_color="#ffffff", hover_color=AppleTheme.AZURE_HOVER,
            corner_radius=AppleTheme.RADIUS_PILL, font=AppleTheme.FONT_BUTTON, command=self._copy_all
        )
        btn_copy.pack(side="right")

    def _copy_all(self):
        self.clipboard_clear()
        self.clipboard_append(self.app_state.preview_text)
        app_logger.info("UI: Copied preview text to clipboard")

    def _build_diff(self):
        if not self.app_state.before_after_data:
            ctk.CTkLabel(self.tab_diff, text="Нет данных для сравнения", font=AppleTheme.FONT_BODY).pack()
            return

        paths = [d['path'] for d in self.app_state.before_after_data]
        self.cmb_diff = ctk.CTkComboBox(self.tab_diff, values=paths, command=self._on_diff_select,
                                        font=AppleTheme.FONT_BODY, corner_radius=AppleTheme.RADIUS_SMALL)
        self.cmb_diff.pack(fill="x", pady=AppleTheme.SP_8)

        split_frame = ctk.CTkFrame(self.tab_diff, fg_color=AppleTheme.TRANSPARENT)
        split_frame.pack(fill="both", expand=True)

        txt_opts = {"font": AppleTheme.FONT_CODE_SM, "corner_radius": AppleTheme.RADIUS_SMALL,
                    "fg_color": AppleTheme.CANVAS, "text_color": AppleTheme.INK}
        self.txt_original = ctk.CTkTextbox(split_frame, **txt_opts)
        self.txt_original.pack(side="left", fill="both", expand=True, padx=AppleTheme.SP_4)

        self.txt_processed = ctk.CTkTextbox(split_frame, **txt_opts)
        self.txt_processed.pack(side="left", fill="both", expand=True, padx=AppleTheme.SP_4)

        self.cmb_diff.set(paths[0])
        self._on_diff_select(paths[0])

    def _on_diff_select(self, path):
        data = next((d for d in self.app_state.before_after_data if d['path'] == path), None)
        if data:
            self.txt_original.configure(state="normal")
            self.txt_original.delete("1.0", "end")
            self.txt_original.insert("1.0", "--- ORIGINAL ---\n\n" + data['original'])
            self.txt_original.configure(state="disabled")

            self.txt_processed.configure(state="normal")
            self.txt_processed.delete("1.0", "end")
            self.txt_processed.insert("1.0", "--- PROCESSED (MINIFIED/SKELETON) ---\n\n" + data['processed'])
            self.txt_processed.configure(state="disabled")

    def _build_history(self):
        if not self.app_state.preview_history:
            ctk.CTkLabel(self.tab_history, text="История пуста", font=AppleTheme.FONT_BODY).pack()
            return

        frame = ctk.CTkFrame(self.tab_history, fg_color=AppleTheme.TRANSPARENT)
        frame.pack(fill="both", expand=True)

        self.hist_list = ctk.CTkScrollableFrame(frame, width=200, fg_color=AppleTheme.CANVAS,
                                                corner_radius=AppleTheme.RADIUS_SMALL)
        self.hist_list.pack(side="left", fill="y", padx=AppleTheme.SP_8)

        self.hist_txt = ctk.CTkTextbox(frame, font=AppleTheme.FONT_CODE_SM, corner_radius=AppleTheme.RADIUS_SMALL,
                                       fg_color=AppleTheme.CANVAS, text_color=AppleTheme.INK)
        self.hist_txt.pack(side="left", fill="both", expand=True)

        for item in self.app_state.preview_history:
            btn = ctk.CTkButton(
                self.hist_list, text=f"{item['time']} ({item['tokens']} tk)",
                fg_color=AppleTheme.TRANSPARENT, hover_color=AppleTheme.FOG, text_color=AppleTheme.INK,
                font=AppleTheme.FONT_BODY_SM, corner_radius=AppleTheme.RADIUS_PILL,
                command=lambda t=item['text']: self._show_hist_text(t)
            )
            btn.pack(fill="x", pady=AppleTheme.SP_4)

    def _show_hist_text(self, text):
        self.hist_txt.configure(state="normal")
        self.hist_txt.delete("1.0", "end")
        self.hist_txt.insert("1.0", text)
        self.hist_txt.configure(state="disabled")

    def update_data(self, new_state):
        self.app_state = new_state
        self.txt_preview.configure(state="normal")
        self.txt_preview.delete("1.0", "end")
        self.txt_preview.insert("1.0", self.app_state.preview_text)
        self.txt_preview.configure(state="disabled")

        if hasattr(self, 'cmb_diff') and self.app_state.before_after_data:
            paths = [d['path'] for d in self.app_state.before_after_data]
            self.cmb_diff.configure(values=paths)
            current = self.cmb_diff.get()
            if current not in paths:
                self.cmb_diff.set(paths[0])
                self._on_diff_select(paths[0])
            else:
                self._on_diff_select(current)

        if hasattr(self, 'hist_list'):
            for widget in self.hist_list.winfo_children():
                widget.destroy()
            for item in self.app_state.preview_history:
                btn = ctk.CTkButton(
                    self.hist_list, text=f"{item['time']} ({item['tokens']} tk)",
                    fg_color=AppleTheme.TRANSPARENT, hover_color=AppleTheme.FOG, text_color=AppleTheme.INK,
                    font=AppleTheme.FONT_BODY_SM, corner_radius=AppleTheme.RADIUS_PILL,
                    command=lambda t=item['text']: self._show_hist_text(t)
                )
                btn.pack(fill="x", pady=AppleTheme.SP_4)

    def _close(self):
        self.on_close_callback()
        self.destroy()


class InputTextDialog(ctk.CTkToplevel):
    def __init__(self, parent, title: str, placeholder: str):
        super().__init__(parent)
        self.title(title)
        self.geometry(AppleTheme.WIN_INPUT)
        self.result = None
        self.transient(parent)
        self.grab_set()
        self.configure(fg_color=AppleTheme.CANVAS)

        self.textbox = ctk.CTkTextbox(
            self, font=AppleTheme.FONT_CODE_SM, corner_radius=AppleTheme.RADIUS_SMALL,
            fg_color=AppleTheme.CARD, text_color=AppleTheme.INK
        )
        self.textbox.pack(fill="both", expand=True, padx=AppleTheme.SP_16, pady=AppleTheme.SP_16)
        self.textbox.insert("1.0", placeholder)

        btn_frame = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
        btn_frame.pack(fill="x", padx=AppleTheme.SP_16, pady=AppleTheme.SP_16)

        ctk.CTkButton(
            btn_frame, text="Применить", command=self._on_ok,
            fg_color=AppleTheme.AZURE, text_color="#ffffff", hover_color=AppleTheme.AZURE_HOVER,
            corner_radius=AppleTheme.RADIUS_PILL, font=AppleTheme.FONT_BUTTON, height=AppleTheme.HEIGHT_BTN_PRIMARY
        ).pack(side="left", expand=True)
        self.wait_window()

    def _on_ok(self):
        self.result = self.textbox.get("1.0", "end-1c")
        self.destroy()

    def get_input(self):
        return self.result


class InteractiveTourDialog(ctk.CTkToplevel):
    def __init__(self, parent, steps: list, on_close_callback):
        super().__init__(parent)
        self.title("Интерактивный тур")
        self.geometry(AppleTheme.WIN_TOUR)
        self.transient(parent)
        self.grab_set()
        self.configure(fg_color=AppleTheme.CARD)

        self.steps = steps
        self.on_close_callback = on_close_callback

        self.current_step = 0
        self.lbl_title = ctk.CTkLabel(self, text="", font=AppleTheme.FONT_DISPLAY, text_color=AppleTheme.INK)
        self.lbl_title.pack(pady=(AppleTheme.SP_40, AppleTheme.SP_24))

        self.txt_desc = ctk.CTkTextbox(
            self, font=AppleTheme.FONT_BODY, wrap="word",
            fg_color=AppleTheme.TRANSPARENT, text_color=AppleTheme.GRAPHITE
        )
        self.txt_desc.pack(fill="both", expand=True, padx=AppleTheme.SP_40, pady=AppleTheme.SP_12)

        self.btn_frame = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
        self.btn_frame.pack(fill="x", padx=AppleTheme.SP_40, pady=AppleTheme.SP_24)

        ghost_opts = {
            "fg_color": AppleTheme.TRANSPARENT, "text_color": AppleTheme.INK,
            "hover_color": AppleTheme.FOG, "corner_radius": AppleTheme.RADIUS_PILL,
            "font": AppleTheme.FONT_BUTTON, "height": AppleTheme.HEIGHT_BTN_PRIMARY
        }

        self.btn_prev = ctk.CTkButton(self.btn_frame, text="⬅ Назад", command=self._prev, **ghost_opts)
        self.btn_prev.pack(side="left")

        self.btn_next = ctk.CTkButton(self.btn_frame, text="Далее ➡", command=self._next, **ghost_opts)
        self.btn_next.pack(side="right")

        self.protocol("WM_DELETE_WINDOW", self._close)

        if self.steps:
            self._update_ui()
        else:
            self._close()

    def _update_ui(self):
        step = self.steps[self.current_step]
        self.lbl_title.configure(text=step.get("title", ""))
        self.txt_desc.configure(state="normal")
        self.txt_desc.delete("1.0", "end")
        self.txt_desc.insert("1.0", step.get("text", ""))
        self.txt_desc.configure(state="disabled")

        self.btn_prev.configure(state="normal" if self.current_step > 0 else "disabled")

        if self.current_step == len(self.steps) - 1:
            self.btn_next.configure(
                text="Начать работу 🚀",
                fg_color=AppleTheme.AZURE,
                text_color="#ffffff",
                hover_color=AppleTheme.AZURE_HOVER
            )
        else:
            self.btn_next.configure(
                text="Далее ➡",
                fg_color=AppleTheme.FOG,
                text_color=AppleTheme.INK,
                hover_color=AppleTheme.BORDER
            )

    def _prev(self):
        if self.current_step > 0:
            self.current_step -= 1
            self._update_ui()

    def _next(self):
        if self.current_step < len(self.steps) - 1:
            self.current_step += 1
            self._update_ui()
        else:
            self._close()

    def _close(self):
        self.on_close_callback()
        self.destroy()