import customtkinter as ctk
from ...utils.config import PRESETS, PROMPT_PRESETS


class Sidebar(ctk.CTkFrame):
    def __init__(self, parent, controller, on_settings_change_callback):
        super().__init__(parent, width=320, corner_radius=0)
        self.controller = controller
        self.on_settings_change = on_settings_change_callback

        self.grid_rowconfigure(2, weight=1)
        self._init_header()
        self._init_tabs()

    def _init_header(self):
        ctk.CTkLabel(self, text="CodeContext AI", font=ctk.CTkFont(size=20, weight="bold")).grid(row=0, column=0,
                                                                                                 padx=20, pady=20)

    def _init_tabs(self):
        self.tab_view = ctk.CTkTabview(self)
        self.tab_view.grid(row=1, column=0, padx=10, pady=10, sticky="nsew")

        self.tab_run = self.tab_view.add("Run")
        self.tab_prompt = self.tab_view.add("Prompt")
        self.tab_settings = self.tab_view.add("Settings")

        self._build_run_tab()
        self._build_prompt_tab()
        self._build_settings_tab()

        ctk.CTkLabel(self, text="v5.2 File Tree", text_color="gray").grid(row=2, column=0, sticky="s", pady=10)

    def _build_run_tab(self):
        t = self.tab_run

        ctk.CTkLabel(t, text="Пресет файлов:").pack(anchor="w", pady=(5, 0))
        self.cmb_preset = ctk.CTkComboBox(t, values=list(PRESETS.keys()), command=self._on_apply_preset)
        self.cmb_preset.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(t, text="Расширения:").pack(anchor="w")
        self.entry_ext = ctk.CTkEntry(t)
        self.entry_ext.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(t, text="Игнор папок:").pack(anchor="w")
        self.entry_ign = ctk.CTkEntry(t)
        self.entry_ign.pack(fill="x", pady=(0, 10))

        self.chk_git = ctk.CTkCheckBox(t, text="Только Git Changes")
        self.chk_git.pack(anchor="w", pady=5)

        self.chk_gitignore = ctk.CTkCheckBox(t, text="Учитывать .gitignore")
        self.chk_gitignore.pack(anchor="w", pady=5)

        self.chk_tree = ctk.CTkCheckBox(t, text="Дерево файлов")
        self.chk_tree.pack(anchor="w", pady=5)

        self.chk_dependencies = ctk.CTkCheckBox(t, text="Карта зависимостей (Beta)")
        self.chk_dependencies.pack(anchor="w", pady=5)

        btn_frame = ctk.CTkFrame(t, fg_color="transparent")
        btn_frame.pack(fill="x", pady=(20, 5))

        self.btn_add_folder = ctk.CTkButton(btn_frame, text="+ Папка", width=140)
        self.btn_add_folder.pack(side="left", padx=(0, 5))

        self.btn_add_github = ctk.CTkButton(btn_frame, text="+ GitHub", width=140, fg_color="#333", hover_color="#444")
        self.btn_add_github.pack(side="right")

        # НОВАЯ КНОПКА
        self.btn_scan = ctk.CTkButton(t, text="🔍 Сканировать (Preview)",
                                      fg_color="gray30",
                                      command=self._on_scan_click)
        self.btn_scan.pack(fill="x", pady=(10, 0))

        self.btn_clear = ctk.CTkButton(t, text="Очистить", fg_color="transparent", border_width=1)
        self.btn_clear.pack(fill="x", pady=5)

    def _build_prompt_tab(self):
        t = self.tab_prompt

        ctk.CTkLabel(t, text="Выберите пресет промпта:").pack(anchor="w", pady=(5, 0))
        self.cmb_prompt_presets = ctk.CTkComboBox(t, values=list(PROMPT_PRESETS.keys()),
                                                  command=self._on_prompt_preset_change)
        self.cmb_prompt_presets.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(t, text="Системный промпт:").pack(anchor="w")
        self.txt_system_prompt = ctk.CTkTextbox(t)
        self.txt_system_prompt.pack(fill="both", expand=True, pady=5)
        self.txt_system_prompt.bind("<KeyRelease>", self._on_prompt_type)

    def _build_settings_tab(self):
        t = self.tab_settings

        cli_frame = ctk.CTkFrame(t)
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

        win_frame = ctk.CTkFrame(t)
        win_frame.pack(fill="x", pady=10)

        ctk.CTkLabel(win_frame, text="Интеграция с Windows", font=ctk.CTkFont(weight="bold")).pack(pady=5)
        self.btn_install_ctx = ctk.CTkButton(win_frame, text="Установить в меню", fg_color="green")
        self.btn_install_ctx.pack(fill="x", padx=10, pady=5)
        self.btn_remove_ctx = ctk.CTkButton(win_frame, text="Удалить из меню", fg_color="red")
        self.btn_remove_ctx.pack(fill="x", padx=10, pady=5)

        self.btn_save = ctk.CTkButton(t, text="💾 Сохранить настройки")
        self.btn_save.pack(fill="x", pady=10)
        self.btn_reset = ctk.CTkButton(t, text="Сбросить все", fg_color="gray")
        self.btn_reset.pack(fill="x", pady=5)

    def _on_apply_preset(self, choice):
        self.controller.apply_preset(choice)

    def _on_prompt_preset_change(self, choice):
        prompt_text = PROMPT_PRESETS.get(choice)
        if choice != "Custom" and prompt_text is not None:
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", prompt_text)
            self.on_settings_change()

    def _on_prompt_type(self, event=None):
        if self.cmb_prompt_presets.get() != "Custom":
            self.cmb_prompt_presets.set("Custom")

    def _on_scan_click(self):
        """Вызов сканирования без копирования"""
        self.controller.update_settings(self.get_settings())
        self.controller.scan_only()

    @staticmethod
    def _set_check(chk, val):
        if val:
            chk.select()
        else:
            chk.deselect()

    def update_ui(self, settings):
        if self.entry_ext.get() != settings.extensions:
            self.entry_ext.delete(0, "end")
            self.entry_ext.insert(0, settings.extensions)

        if self.entry_ign.get() != settings.ignored_paths:
            self.entry_ign.delete(0, "end")
            self.entry_ign.insert(0, settings.ignored_paths)

        checkboxes = [
            (self.chk_tree, settings.include_tree),
            (self.chk_dependencies, settings.include_dependencies),
            (self.chk_git, settings.use_git),
            (self.chk_gitignore, settings.use_gitignore),
            (self.chk_cli_minify, settings.cli_minify),
            (self.chk_cli_comments, settings.cli_remove_comments),
            (self.chk_cli_secrets, settings.cli_remove_secrets),
            (self.chk_cli_tree, settings.cli_include_tree),
            (self.chk_cli_skeleton, settings.cli_skeleton_mode),
            (self.chk_cli_gitignore, settings.cli_use_gitignore)
        ]
        for chk, val in checkboxes:
            self._set_check(chk, val)

        self.cmb_cli_format.set(settings.cli_format)

        current_prompt = self.txt_system_prompt.get("1.0", "end-1c")
        if current_prompt != settings.system_prompt and not self.txt_system_prompt.focus_get():
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", settings.system_prompt)

        found = False
        for name, text in PROMPT_PRESETS.items():
            if name != "Custom" and text.strip() == settings.system_prompt.strip():
                self.cmb_prompt_presets.set(name)
                found = True
                break
        if not found:
            self.cmb_prompt_presets.set("Custom")

    def get_settings(self):
        return {
            'extensions': self.entry_ext.get(),
            'ignored_paths': self.entry_ign.get(),
            'include_tree': bool(self.chk_tree.get()),
            'include_dependencies': bool(self.chk_dependencies.get()),
            'use_git': bool(self.chk_git.get()),
            'use_gitignore': bool(self.chk_gitignore.get()),
            'system_prompt': self.txt_system_prompt.get("1.0", "end-1c"),
            'cli_minify': bool(self.chk_cli_minify.get()),
            'cli_remove_comments': bool(self.chk_cli_comments.get()),
            'cli_remove_secrets': bool(self.chk_cli_secrets.get()),
            'cli_include_tree': bool(self.chk_cli_tree.get()),
            'cli_skeleton_mode': bool(self.chk_cli_skeleton.get()),
            'cli_use_gitignore': bool(self.chk_cli_gitignore.get()),
            'cli_format': self.cmb_cli_format.get()
        }

    def set_loading(self, is_loading):
        state = "disabled" if is_loading else "normal"
        self.tab_view.configure(state=state)