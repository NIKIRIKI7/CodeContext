import customtkinter as ctk
from tkinter import filedialog


class EditFolderDialog(ctk.CTkToplevel):
    def __init__(self, parent, initial_path: str):
        super().__init__(parent)
        self.title("Редактирование")
        self.geometry("500x160")
        self.resizable(False, False)
        self.result = None
        self.transient(parent)
        self.grab_set()
        self.update_idletasks()
        try:
            x = parent.winfo_x() + (parent.winfo_width() // 2) - (500 // 2)
            y = parent.winfo_y() + (parent.winfo_height() // 2) - (160 // 2)
            self.geometry(f"+{x}+{y}")
        except Exception:
            pass
        ctk.CTkLabel(self, text="Измените путь:", font=("Arial", 14)).pack(pady=(20, 5))
        self.input_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.input_frame.pack(fill="x", padx=20, pady=10)
        self.entry = ctk.CTkEntry(self.input_frame)
        self.entry.pack(side="left", fill="x", expand=True, padx=(0, 10))
        self.entry.insert(0, initial_path)
        self.btn_browse = ctk.CTkButton(self.input_frame, text="📁", width=40, command=self._on_browse)
        self.btn_browse.pack(side="right")
        self.btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.btn_frame.pack(fill="x", padx=20, pady=5)
        self.btn_ok = ctk.CTkButton(self.btn_frame, text="OK", width=100, command=self._on_ok)
        self.btn_ok.pack(side="left", expand=True)
        self.btn_cancel = ctk.CTkButton(self.btn_frame, text="Отмена", width=100, fg_color="transparent",
                                        border_width=1, command=self.destroy)
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
    """Глубокий предпросмотр (Context / Diff / History)"""

    def __init__(self, parent, state, on_close_callback):
        super().__init__(parent)
        self.title("Deep Preview")
        self.geometry("1000x800")
        self.transient(parent)
        self.on_close_callback = on_close_callback
        self.state = state

        self.tabview = ctk.CTkTabview(self)
        self.tabview.pack(fill="both", expand=True, padx=10, pady=10)

        self.tab_preview = self.tabview.add("📝 Контекст")
        self.tab_diff = self.tabview.add("⚖️ До/После")
        self.tab_history = self.tabview.add("🕒 История")

        self._build_preview()
        self._build_diff()
        self._build_history()

        self.protocol("WM_DELETE_WINDOW", self._close)

    def _build_preview(self):
        self.txt_preview = ctk.CTkTextbox(self.tab_preview, wrap="word", font=("Consolas", 13))
        self.txt_preview.pack(fill="both", expand=True, pady=(0, 10))
        self.txt_preview.insert("1.0", self.state.preview_text)
        self.txt_preview.configure(state="disabled")

        btn_copy = ctk.CTkButton(self.tab_preview, text="📋 Копировать всё", command=self._copy_all)
        btn_copy.pack(side="right")

    def _copy_all(self):
        self.clipboard_clear()
        self.clipboard_append(self.state.preview_text)

    def _build_diff(self):
        if not self.state.before_after_data:
            ctk.CTkLabel(self.tab_diff, text="Нет данных для сравнения").pack()
            return

        paths = [d['path'] for d in self.state.before_after_data]
        self.cmb_diff = ctk.CTkComboBox(self.tab_diff, values=paths, command=self._on_diff_select)
        self.cmb_diff.pack(fill="x", pady=5)

        split_frame = ctk.CTkFrame(self.tab_diff)
        split_frame.pack(fill="both", expand=True)

        self.txt_original = ctk.CTkTextbox(split_frame, font=("Consolas", 12))
        self.txt_original.pack(side="left", fill="both", expand=True, padx=2)

        self.txt_processed = ctk.CTkTextbox(split_frame, font=("Consolas", 12))
        self.txt_processed.pack(side="left", fill="both", expand=True, padx=2)

        self.cmb_diff.set(paths[0])
        self._on_diff_select(paths[0])

    def _on_diff_select(self, path):
        data = next((d for d in self.state.before_after_data if d['path'] == path), None)
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
        if not self.state.preview_history:
            ctk.CTkLabel(self.tab_history, text="История пуста").pack()
            return

        frame = ctk.CTkFrame(self.tab_history)
        frame.pack(fill="both", expand=True)

        self.hist_list = ctk.CTkScrollableFrame(frame, width=200)
        self.hist_list.pack(side="left", fill="y", padx=5)

        self.hist_txt = ctk.CTkTextbox(frame, font=("Consolas", 12))
        self.hist_txt.pack(side="left", fill="both", expand=True)

        for item in self.state.preview_history:
            btn = ctk.CTkButton(self.hist_list, text=f"{item['time']} ({item['tokens']} tk)",
                                command=lambda t=item['text']: self._show_hist_text(t))
            btn.pack(fill="x", pady=2)

    def _show_hist_text(self, text):
        self.hist_txt.configure(state="normal")
        self.hist_txt.delete("1.0", "end")
        self.hist_txt.insert("1.0", text)
        self.hist_txt.configure(state="disabled")

    def _close(self):
        self.on_close_callback()
        self.destroy()


class InputTextDialog(ctk.CTkToplevel):
    """Универсальный диалог для ввода большого текста (Патчи / Логи)"""

    def __init__(self, parent, title: str, placeholder: str):
        super().__init__(parent)
        self.title(title)
        self.geometry("600x500")
        self.result = None
        self.transient(parent)
        self.grab_set()

        self.textbox = ctk.CTkTextbox(self, font=("Consolas", 12))
        self.textbox.pack(fill="both", expand=True, padx=10, pady=10)
        self.textbox.insert("1.0", placeholder)

        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(fill="x", padx=10, pady=10)
        ctk.CTkButton(btn_frame, text="Применить", command=self._on_ok).pack(side="left", expand=True)
        self.wait_window()

    def _on_ok(self):
        self.result = self.textbox.get("1.0", "end-1c")
        self.destroy()

    def get_input(self):
        return self.result


class InteractiveTourDialog(ctk.CTkToplevel):
    """Интерактивный тур с детальной инструкцией по применению"""

    def __init__(self, parent):
        super().__init__(parent)
        self.title("Интерактивный тур - Как пользоваться CodeContext")
        self.geometry("700x500")
        self.transient(parent)
        self.grab_set()

        try:
            x = parent.winfo_x() + (parent.winfo_width() // 2) - (700 // 2)
            y = parent.winfo_y() + (parent.winfo_height() // 2) - (500 // 2)
            self.geometry(f"+{x}+{y}")
        except Exception:
            pass

        self.steps = [
            {
                "title": "👋 Добро пожаловать в CodeContext!",
                "text": "CodeContext AI собирает ваш проект в единый, оптимизированный текстовый промпт для нейросетей.\n\nЭто позволяет ChatGPT, Claude или Cursor 'понять' всю архитектуру вашего проекта за секунды, избегая лимитов по токенам."
            },
            {
                "title": "📂 Шаг 1: Добавление файлов",
                "text": "1. Перетащите папку проекта (Drag & Drop) прямо в окно программы или нажмите кнопку '+ Папка'.\n2. Отметьте нужные файлы галочками в дереве.\n\n💡 Фишка: Файлы из .gitignore исключаются автоматически, а измененные в Git файлы подсвечиваются цветами (желтый/зеленый)!"
            },
            {
                "title": "⚙️ Шаг 2: Экономия контекста",
                "text": "Укажите настройки перед сборкой в панели ниже:\n\n• Minify - уберет пустые строки.\n• No Comments - вырежет все комментарии.\n• Skeleton ☠️ - оставит ТОЛЬКО названия функций/классов, удалив их логику (мастхэв для огромных проектов!)."
            },
            {
                "title": "💬 Шаг 3: Магия промптов (Code Patcher)",
                "text": "Перейдите на вкладку 'Prompt' в боковом меню и выберите пресет 'Code Patcher (JSON)'.\n\nЭтот системный промпт заставит нейросеть возвращать ответы в строгом формате JSON (с поддержкой 7 действий: replace, create, delete, append и т.д.). Вы сэкономите часы на копировании-вставке!"
            },
            {
                "title": "🧩 Шаг 4: Применение патчей от LLM",
                "text": "Получили ответ от ИИ с кодом?\n\n1. Скопируйте ответ (с массивом JSON).\n2. На вкладке 'Run' нажмите зеленую кнопку '🧩 Применить JSON-патч от LLM'.\n3. Вставьте текст и нажмите 'Применить'.\n\nУмный алгоритм сам найдет нужные файлы (игнорируя пробелы) и обновит точечные строки!"
            },
            {
                "title": "👀 Шаг 5: Продвинутый Предпросмотр",
                "text": "Нажмите кнопку '👀 Предпросмотр' (или Ctrl+Enter) перед сохранением в буфер обмена!\n\nВ открывшемся окне можно:\n- Посмотреть итоговый сгенерированный текст.\n- Открыть вкладку 'До/После', чтобы сравнить оригинальный код с ужатым (Minify/Skeleton).\n- Найти прошлые генерации на вкладке 'История'."
            }
        ]
        self.current_step = 0

        self.lbl_title = ctk.CTkLabel(self, text="", font=("Arial", 22, "bold"))
        self.lbl_title.pack(pady=(20, 15))

        self.txt_desc = ctk.CTkTextbox(self, font=("Consolas", 15), wrap="word", fg_color="transparent")
        self.txt_desc.pack(fill="both", expand=True, padx=30, pady=10)

        self.btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.btn_frame.pack(fill="x", padx=30, pady=20)

        self.btn_prev = ctk.CTkButton(self.btn_frame, text="⬅ Назад", command=self._prev)
        self.btn_prev.pack(side="left")

        self.btn_next = ctk.CTkButton(self.btn_frame, text="Далее ➡", command=self._next)
        self.btn_next.pack(side="right")

        self._update_ui()

    def _update_ui(self):
        step = self.steps[self.current_step]
        self.lbl_title.configure(text=step["title"])

        self.txt_desc.configure(state="normal")
        self.txt_desc.delete("1.0", "end")
        self.txt_desc.insert("1.0", step["text"])
        self.txt_desc.configure(state="disabled")

        self.btn_prev.configure(state="normal" if self.current_step > 0 else "disabled")

        if self.current_step == len(self.steps) - 1:
            self.btn_next.configure(text="Начать работу 🚀", fg_color="#27694a", hover_color="#1d4f37")
        else:
            self.btn_next.configure(text="Далее ➡", fg_color=["#3B8ED0", "#1F6AA5"], hover_color=["#36719F", "#144870"])

    def _prev(self):
        if self.current_step > 0:
            self.current_step -= 1
            self._update_ui()

    def _next(self):
        if self.current_step < len(self.steps) - 1:
            self.current_step += 1
            self._update_ui()
        else:
            self.destroy()