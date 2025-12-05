import os
import customtkinter as ctk
from tkinter import filedialog, messagebox
from tkinterdnd2 import TkinterDnD, DND_FILES
from ..store.store import Store
from ..controllers.main_controller import MainController
from ..utils.config import PRESETS, PROMPT_PRESETS

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")


class MainWindow(ctk.CTk, TkinterDnD.DnDWrapper):
    """
    Главное окно приложения (View).
    Добавлена поддержка Drag & Drop через tkinterdnd2.
    """

    def __init__(self, store: Store, controller: MainController):
        # 1. Инициализация CTk
        super().__init__()

        # 2. Инициализация DnD (Критически важно вызвать это сразу)
        self.TkdndVersion = TkinterDnD._require(self)

        self.store = store
        self.controller = controller
        self.title("CodeContext AI - Clean Architecture")
        self.geometry("1150x850")

        self.unsubscribe = self.store.subscribe(self._on_store_changed)
        self._init_ui()

        # 3. Регистрируем DnD на все окно целиком (самый надежный способ)
        self.drop_target_register(DND_FILES)
        self.dnd_bind('<<Drop>>', self._on_drop)

        self.controller.load_initial_settings()

    def _init_ui(self):
        """Создание UI элементов"""
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # === SIDEBAR ===
        sidebar = ctk.CTkFrame(self, width=320, corner_radius=0)
        sidebar.grid(row=0, column=0, sticky="nsew")
        sidebar.grid_rowconfigure(2, weight=1)

        ctk.CTkLabel(sidebar, text="CodeContext AI", font=ctk.CTkFont(size=20, weight="bold")).grid(row=0, column=0,
                                                                                                    padx=20, pady=20)

        self.tab_view = ctk.CTkTabview(sidebar)
        self.tab_view.grid(row=1, column=0, padx=10, pady=10, sticky="nsew")

        self.tab_run = self.tab_view.add("Run")
        self.tab_prompt = self.tab_view.add("Prompt")
        self.tab_settings = self.tab_view.add("Settings")

        # --- Tab RUN ---
        ctk.CTkLabel(self.tab_run, text="Пресет файлов:").pack(anchor="w", pady=(5, 0))
        self.cmb_preset = ctk.CTkComboBox(self.tab_run, values=list(PRESETS.keys()), command=self._on_apply_preset)
        self.cmb_preset.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(self.tab_run, text="Расширения:").pack(anchor="w")
        self.entry_ext = ctk.CTkEntry(self.tab_run)
        self.entry_ext.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(self.tab_run, text="Игнор папок:").pack(anchor="w")
        self.entry_ign = ctk.CTkEntry(self.tab_run)
        self.entry_ign.pack(fill="x", pady=(0, 10))

        self.chk_git = ctk.CTkCheckBox(self.tab_run, text="Только Git Changes")
        self.chk_git.pack(anchor="w", pady=5)

        self.chk_gitignore = ctk.CTkCheckBox(self.tab_run, text="Учитывать .gitignore")
        self.chk_gitignore.pack(anchor="w", pady=5)

        self.chk_tree = ctk.CTkCheckBox(self.tab_run, text="Дерево файлов")
        self.chk_tree.pack(anchor="w", pady=5)

        # Buttons Frame
        btn_frame = ctk.CTkFrame(self.tab_run, fg_color="transparent")
        btn_frame.pack(fill="x", pady=(20, 5))

        ctk.CTkButton(btn_frame, text="+ Папка", width=140, command=self._on_add_folder).pack(side="left", padx=(0, 5))
        ctk.CTkButton(btn_frame, text="+ GitHub", width=140, command=self._on_add_github, fg_color="#2B2B2B",
                      border_width=1).pack(side="right", padx=(5, 0))

        ctk.CTkButton(self.tab_run, text="Очистить", command=self._on_clear_folders, fg_color="transparent",
                      border_width=1).pack(fill="x", pady=5)

        # --- Tab PROMPT ---
        ctk.CTkLabel(self.tab_prompt, text="Выберите пресет промпта:").pack(anchor="w", pady=(5, 0))
        self.cmb_prompt_presets = ctk.CTkComboBox(
            self.tab_prompt,
            values=list(PROMPT_PRESETS.keys()),
            command=self._on_prompt_preset_change
        )
        self.cmb_prompt_presets.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(self.tab_prompt, text="Системный промпт:").pack(anchor="w")
        self.txt_system_prompt = ctk.CTkTextbox(self.tab_prompt)
        self.txt_system_prompt.pack(fill="both", expand=True, pady=5)
        self.txt_system_prompt.bind("<KeyRelease>", self._on_prompt_type)

        # --- Tab SETTINGS (CLI & WIN) ---
        cli_frame = ctk.CTkFrame(self.tab_settings)
        cli_frame.pack(fill="x", pady=5)
        ctk.CTkLabel(cli_frame, text="Настройки CLI", font=ctk.CTkFont(weight="bold")).pack(pady=5)

        self.chk_cli_minify = ctk.CTkCheckBox(cli_frame, text="Minify (сжать)")
        self.chk_cli_minify.pack(anchor="w", padx=10, pady=2)

        self.chk_cli_comments = ctk.CTkCheckBox(cli_frame, text="Удалять комментарии")
        self.chk_cli_comments.pack(anchor="w", padx=10, pady=2)

        self.chk_cli_secrets = ctk.CTkCheckBox(cli_frame, text="Скрывать секреты")
        self.chk_cli_secrets.pack(anchor="w", padx=10, pady=2)

        self.chk_cli_tree = ctk.CTkCheckBox(cli_frame, text="Добавлять дерево файлов")
        self.chk_cli_tree.pack(anchor="w", padx=10, pady=2)

        self.chk_cli_skeleton = ctk.CTkCheckBox(cli_frame, text="Skeleton Mode")
        self.chk_cli_skeleton.pack(anchor="w", padx=10, pady=2)

        self.chk_cli_gitignore = ctk.CTkCheckBox(cli_frame, text="Учитывать .gitignore")
        self.chk_cli_gitignore.pack(anchor="w", padx=10, pady=2)

        ctk.CTkLabel(cli_frame, text="Формат вывода:").pack(anchor="w", padx=10, pady=(5, 0))
        self.cmb_cli_format = ctk.CTkComboBox(cli_frame, values=["plain", "markdown", "xml"])
        self.cmb_cli_format.pack(fill="x", padx=10, pady=5)

        win_frame = ctk.CTkFrame(self.tab_settings)
        win_frame.pack(fill="x", pady=10)
        ctk.CTkLabel(win_frame, text="Интеграция с Windows", font=ctk.CTkFont(weight="bold")).pack(pady=5)

        ctk.CTkButton(win_frame, text="Установить в меню", fg_color="green", command=self._on_install_context).pack(
            fill="x", padx=10, pady=5)
        ctk.CTkButton(win_frame, text="Удалить из меню", fg_color="red", command=self._on_remove_context).pack(fill="x",
                                                                                                               padx=10,
                                                                                                               pady=5)

        ctk.CTkButton(self.tab_settings, text="💾 Сохранить настройки", command=self._on_save_settings).pack(fill="x",
                                                                                                            pady=10)
        ctk.CTkButton(self.tab_settings, text="Сбросить все", fg_color="gray", command=self._on_reset_settings).pack(
            fill="x", pady=5)

        ctk.CTkLabel(self.tab_settings, text="v4.4 Drag & Drop Fixed", text_color="gray").pack(side="bottom", pady=10)

        # === MAIN CONTENT ===
        main_frame = ctk.CTkFrame(self, fg_color="transparent")
        main_frame.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)
        main_frame.grid_rowconfigure(3, weight=1)
        main_frame.grid_columnconfigure(0, weight=1)

        # 1. Folders List
        self.scroll_folders = ctk.CTkScrollableFrame(main_frame, height=120,
                                                     label_text="Источники (Перетащите папки сюда)")
        self.scroll_folders.grid(row=0, column=0, sticky="ew", pady=(0, 10))

        # 2. Options Bar
        opts = ctk.CTkFrame(main_frame, fg_color="transparent")
        opts.grid(row=1, column=0, sticky="ew", pady=(0, 10))

        self.chk_minify = ctk.CTkCheckBox(opts, text="Minify")
        self.chk_minify.pack(side="left", padx=10)

        self.chk_comments = ctk.CTkCheckBox(opts, text="No Comments")
        self.chk_comments.pack(side="left", padx=10)

        self.chk_secrets = ctk.CTkCheckBox(opts, text="No Secrets")
        self.chk_secrets.pack(side="left", padx=10)

        self.chk_skeleton = ctk.CTkCheckBox(opts, text="Skeleton ☠️")
        self.chk_skeleton.pack(side="left", padx=10)

        self.seg_format = ctk.CTkSegmentedButton(opts, values=["markdown", "xml", "plain"])
        self.seg_format.pack(side="right")
        self.seg_format.set("markdown")

        # 3. Action Buttons
        btns = ctk.CTkFrame(main_frame)
        btns.grid(row=2, column=0, sticky="ew", pady=(0, 10))

        ctk.CTkButton(btns, text="В Буфер", command=lambda: self._on_run("clipboard")).pack(side="left", expand=True,
                                                                                            fill="x", padx=5, pady=5)
        ctk.CTkButton(btns, text="В Файл", command=lambda: self._on_run("file")).pack(side="left", expand=True,
                                                                                      fill="x", padx=5, pady=5)
        ctk.CTkButton(btns, text="В PDF", command=lambda: self._on_run("pdf")).pack(side="left", expand=True, fill="x",
                                                                                    padx=5, pady=5)

        # 4. Logs
        self.txt_log = ctk.CTkTextbox(main_frame, font=("Consolas", 12))
        self.txt_log.grid(row=3, column=0, sticky="nsew")

        # 5. Status Bar
        status_frame = ctk.CTkFrame(main_frame, height=30)
        status_frame.grid(row=4, column=0, sticky="ew", pady=(10, 0))

        self.progress_bar = ctk.CTkProgressBar(status_frame)
        self.progress_bar.pack(fill="x", side="top")
        self.progress_bar.set(0)

        self.lbl_status = ctk.CTkLabel(status_frame, text="Idle", text_color="gray")
        self.lbl_status.pack(side="left", padx=5)

        self.lbl_tokens = ctk.CTkLabel(status_frame, text="Tokens: 0", text_color="gray")
        self.lbl_tokens.pack(side="right", padx=5)

    # <--- HANDLER ---
    def _on_drop(self, event):
        """Обработка перетаскивания файлов/папок"""
        if not event.data:
            return

        try:
            # Парсим пути, учитывая пробелы и фигурные скобки Windows
            raw_data = event.data
            paths = self.tk.splitlist(raw_data)

            count = 0
            for path in paths:
                # Очистка от лишних кавычек или скобок, если tk.splitlist пропустил
                path = path.strip('{}')

                if os.path.isdir(path):
                    self.controller.add_folder(path)
                    count += 1
                elif os.path.isfile(path):
                    # Если перетащили файл, берем его родительскую папку
                    # или можно добавить сам файл, если доработать логику
                    folder = os.path.dirname(path)
                    self.controller.add_folder(folder)
                    count += 1

            if count > 0:
                self.lbl_status.configure(text=f"Добавлено: {count}")

        except Exception as e:
            print(f"Drop error: {e}")
            self.lbl_status.configure(text="Ошибка Drag&Drop")

    # ---------------------------

    def _on_store_changed(self, state):
        """Обновление UI при изменении данных в Store"""
        if self.entry_ext.get() != state.settings.extensions:
            self.entry_ext.delete(0, "end")
            self.entry_ext.insert(0, state.settings.extensions)

        if self.entry_ign.get() != state.settings.ignored_paths:
            self.entry_ign.delete(0, "end")
            self.entry_ign.insert(0, state.settings.ignored_paths)

        self._set_check(self.chk_minify, state.settings.minify)
        self._set_check(self.chk_comments, state.settings.remove_comments)
        self._set_check(self.chk_secrets, state.settings.remove_secrets)
        self._set_check(self.chk_tree, state.settings.include_tree)
        self._set_check(self.chk_git, state.settings.use_git)
        self._set_check(self.chk_gitignore, state.settings.use_gitignore)
        self._set_check(self.chk_skeleton, state.settings.skeleton_mode)

        current_prompt = self.txt_system_prompt.get("1.0", "end-1c")
        if current_prompt != state.settings.system_prompt and not self.txt_system_prompt.focus_get():
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", state.settings.system_prompt)

        # Match Preset
        found = False
        for name, text in PROMPT_PRESETS.items():
            if name != "Custom" and text.strip() == state.settings.system_prompt.strip():
                self.cmb_prompt_presets.set(name)
                found = True
                break
        if not found:
            self.cmb_prompt_presets.set("Custom")

        # CLI Settings
        self._set_check(self.chk_cli_minify, state.settings.cli_minify)
        self._set_check(self.chk_cli_comments, state.settings.cli_remove_comments)
        self._set_check(self.chk_cli_secrets, state.settings.cli_remove_secrets)
        self._set_check(self.chk_cli_tree, state.settings.cli_include_tree)
        self._set_check(self.chk_cli_skeleton, state.settings.cli_skeleton_mode)
        self._set_check(self.chk_cli_gitignore, state.settings.cli_use_gitignore)
        self.cmb_cli_format.set(state.settings.cli_format)

        # Folders
        if len(self.scroll_folders.winfo_children()) != len(state.selected_folders):
            for w in self.scroll_folders.winfo_children():
                w.destroy()
            for folder in state.selected_folders:
                row = ctk.CTkFrame(self.scroll_folders)
                row.pack(fill="x", pady=2)
                is_temp = folder in state.temp_folders
                prefix = "☁️" if is_temp else "📂"
                ctk.CTkLabel(row, text=f"{prefix} {folder}", anchor="w").pack(side="left", padx=5)

        # Logs & Status
        self.txt_log.configure(state="normal")
        self.txt_log.delete("1.0", "end")
        for log in state.logs:
            self.txt_log.insert("end", f"{log}\n")
        self.txt_log.see("end")
        self.txt_log.configure(state="disabled")

        self.lbl_status.configure(text=state.status_message)
        self.progress_bar.set(state.progress)
        self.lbl_tokens.configure(text=f"Tokens: {state.total_tokens}")

        state_state = "disabled" if state.is_loading else "normal"
        self.tab_view.configure(state=state_state)

    def _set_check(self, chk, val):
        if val:
            chk.select()
        else:
            chk.deselect()

    def _collect_settings_from_ui(self):
        return {
            'extensions': self.entry_ext.get(),
            'ignored_paths': self.entry_ign.get(),
            'minify': bool(self.chk_minify.get()),
            'remove_comments': bool(self.chk_comments.get()),
            'remove_secrets': bool(self.chk_secrets.get()),
            'include_tree': bool(self.chk_tree.get()),
            'skeleton_mode': bool(self.chk_skeleton.get()),
            'use_git': bool(self.chk_git.get()),
            'use_gitignore': bool(self.chk_gitignore.get()),
            'system_prompt': self.txt_system_prompt.get("1.0", "end-1c"),
            'output_format': self.seg_format.get(),
            'cli_minify': bool(self.chk_cli_minify.get()),
            'cli_remove_comments': bool(self.chk_cli_comments.get()),
            'cli_remove_secrets': bool(self.chk_cli_secrets.get()),
            'cli_include_tree': bool(self.chk_cli_tree.get()),
            'cli_skeleton_mode': bool(self.chk_cli_skeleton.get()),
            'cli_use_gitignore': bool(self.chk_cli_gitignore.get()),
            'cli_format': self.cmb_cli_format.get()
        }

    # ... Handlers
    def _on_apply_preset(self, choice):
        self.controller.apply_preset(choice)

    def _on_prompt_preset_change(self, choice):
        prompt_text = PROMPT_PRESETS.get(choice)
        if choice != "Custom" and prompt_text is not None:
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", prompt_text)
            data = self._collect_settings_from_ui()
            self.controller.update_settings(data)

    def _on_prompt_type(self, event):
        if self.cmb_prompt_presets.get() != "Custom":
            self.cmb_prompt_presets.set("Custom")

    def _on_add_folder(self):
        path = filedialog.askdirectory()
        if path:
            self.controller.add_folder(path)

    def _on_add_github(self):
        dialog = ctk.CTkInputDialog(text="Введите URL репозитория:", title="GitHub Clone")
        url = dialog.get_input()
        if url:
            self.controller.add_github_repo(url)

    def _on_clear_folders(self):
        self.controller.clear_folders()

    def _on_install_context(self):
        try:
            success, msg = self.controller.install_context_menu()
            if success:
                messagebox.showinfo("Успех", msg)
            else:
                if "Запрошены права" in msg:
                    messagebox.showinfo("Требуются права", msg)
                else:
                    messagebox.showerror("Ошибка", msg)
        except SystemExit:
            pass

    def _on_remove_context(self):
        try:
            success, msg = self.controller.remove_context_menu()
            if success:
                messagebox.showinfo("Успех", msg)
            else:
                if "Запрошены права" in msg:
                    messagebox.showinfo("Требуются права", msg)
                else:
                    messagebox.showerror("Ошибка", msg)
        except SystemExit:
            pass

    def _on_save_settings(self):
        data = self._collect_settings_from_ui()
        self.controller.update_settings(data)
        self.controller.save_settings()
        messagebox.showinfo("Сохранено", "Настройки успешно сохранены!")

    def _on_reset_settings(self):
        if messagebox.askyesno("Сброс", "Сбросить все настройки?"):
            self.controller.reset_settings()

    def _on_run(self, target):
        data = self._collect_settings_from_ui()
        self.controller.update_settings(data)
        save_path = None
        state = self.store.state
        if target == 'file':
            ext = f".{state.settings.output_format}" if state.settings.output_format != 'plain' else ".txt"
            save_path = filedialog.asksaveasfilename(defaultextension=ext)
            if not save_path: return
        elif target == 'pdf':
            save_path = filedialog.asksaveasfilename(defaultextension=".pdf")
            if not save_path: return
        success, msg = self.controller.start_processing(target, save_path)
        if not success:
            messagebox.showwarning("Внимание", msg)

    def on_closing(self):
        self.controller.clear_folders()
        if self.unsubscribe:
            self.unsubscribe()
        self.destroy()