import customtkinter as ctk
from tkinter import filedialog
from typing import Any
from ...utils.config import PRESETS, PROMPT_PRESETS
from ..theme import AppleTheme


class Sidebar(ctk.CTkFrame):
    def __init__(self, parent: Any, controller: Any, on_settings_change_callback: Any):
        super().__init__(
            parent,
            width=AppleTheme.SIDEBAR_WIDTH,
            corner_radius=AppleTheme.RADIUS_CARD,
            fg_color=AppleTheme.CARD
        )
        self.controller = controller
        self.on_settings_change = on_settings_change_callback

        self.grid_rowconfigure(1, weight=1)
        self.grid_rowconfigure(2, weight=0)

        self._init_header()
        self._init_tabs()

    def _init_header(self):
        lbl = ctk.CTkLabel(self, text="CodeContext AI", font=AppleTheme.FONT_HEADING, text_color=AppleTheme.INK)
        lbl.grid(row=0, column=0, padx=AppleTheme.SP_20, pady=(AppleTheme.SP_24, AppleTheme.SP_12), sticky="w")

    def _init_tabs(self):
        self.tab_view = ctk.CTkTabview(
            self,
            fg_color=AppleTheme.TRANSPARENT,
            segmented_button_selected_color=AppleTheme.FOG,
            segmented_button_selected_hover_color=AppleTheme.BORDER,
            segmented_button_unselected_color=AppleTheme.CARD,  # ИСПРАВЛЕНО: Вместо TRANSPARENT
            segmented_button_unselected_hover_color=AppleTheme.FOG,
            text_color=AppleTheme.INK
        )
        self.tab_view.grid(row=1, column=0, padx=AppleTheme.SP_20, pady=AppleTheme.SP_0, sticky="nsew")

        self.tab_run = self.tab_view.add("Сборка")
        self.tab_prompt = self.tab_view.add("Промпты")
        self.tab_settings = self.tab_view.add("Настройки")

        self._build_run_tab()
        self._build_prompt_tab()
        self._build_settings_tab()

        bottom_frame = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
        bottom_frame.grid(row=2, column=0, sticky="sew", pady=AppleTheme.SP_20)

        lbl_version = ctk.CTkLabel(bottom_frame, text="v5.3 Workspace", font=AppleTheme.FONT_BODY_SM,
                                   text_color=AppleTheme.GRAPHITE)
        lbl_version.pack(side="left", padx=AppleTheme.SP_20)

        btn_tour = ctk.CTkButton(
            bottom_frame, text="Инструкция", width=100, height=AppleTheme.SP_28,
            fg_color=AppleTheme.TRANSPARENT, hover_color=AppleTheme.FOG,
            text_color=AppleTheme.INK, font=AppleTheme.FONT_BODY_SM,
            corner_radius=AppleTheme.RADIUS_PILL, command=self._show_tour
        )
        btn_tour.pack(side="right", padx=AppleTheme.SP_20)

    def _show_tour(self):
        self.controller.show_tour()

    def _build_run_tab(self):
        t_scroll = ctk.CTkScrollableFrame(self.tab_run, fg_color=AppleTheme.TRANSPARENT)
        t_scroll.pack(fill="both", expand=True)
        t = t_scroll

        ws_frame = ctk.CTkFrame(t, fg_color=AppleTheme.TRANSPARENT)
        ws_frame.pack(fill="x", pady=(AppleTheme.SP_0, AppleTheme.SP_16))

        btn_opts = {
            "fg_color": AppleTheme.FOG,
            "text_color": AppleTheme.INK,
            "hover_color": AppleTheme.BORDER,
            "corner_radius": AppleTheme.RADIUS_PILL,
            "font": AppleTheme.FONT_BODY_SM,
            "height": AppleTheme.HEIGHT_BTN_SEC
        }

        self.btn_save_ws = ctk.CTkButton(ws_frame, text="Сохранить", width=110, command=self._on_save_workspace,
                                         **btn_opts)
        self.btn_save_ws.pack(side="left")
        self.btn_load_ws = ctk.CTkButton(ws_frame, text="Загрузить", width=110, command=self._on_load_workspace,
                                         **btn_opts)
        self.btn_load_ws.pack(side="right")

        lbl_opts = {"font": AppleTheme.FONT_BODY_SM, "text_color": AppleTheme.GRAPHITE}
        entry_opts = {"font": AppleTheme.FONT_BODY, "corner_radius": AppleTheme.RADIUS_SMALL}

        ctk.CTkLabel(t, text="Пресет файлов:", **lbl_opts).pack(anchor="w", pady=(AppleTheme.SP_4, AppleTheme.SP_2))
        self.cmb_preset = ctk.CTkComboBox(t, values=list(PRESETS.keys()), command=self._on_apply_preset, **entry_opts)
        self.cmb_preset.pack(fill="x", pady=(AppleTheme.SP_0, AppleTheme.SP_12))

        ctk.CTkLabel(t, text="Расширения:", **lbl_opts).pack(anchor="w")
        self.entry_ext = ctk.CTkEntry(t, **entry_opts)
        self.entry_ext.pack(fill="x", pady=(AppleTheme.SP_0, AppleTheme.SP_12))

        ctk.CTkLabel(t, text="Игнор папок:", **lbl_opts).pack(anchor="w")
        self.entry_ign = ctk.CTkEntry(t, **entry_opts)
        self.entry_ign.pack(fill="x", pady=(AppleTheme.SP_0, AppleTheme.SP_16))

        chk_opts = {"font": AppleTheme.FONT_BODY, "text_color": AppleTheme.INK}
        self.chk_git = ctk.CTkCheckBox(t, text="Только Git Changes", **chk_opts)
        self.chk_git.pack(anchor="w", pady=AppleTheme.SP_6)
        self.chk_gitignore = ctk.CTkCheckBox(t, text="Учитывать .gitignore", **chk_opts)
        self.chk_gitignore.pack(anchor="w", pady=AppleTheme.SP_6)
        self.chk_tree = ctk.CTkCheckBox(t, text="Дерево файлов", **chk_opts)
        self.chk_tree.pack(anchor="w", pady=AppleTheme.SP_6)
        self.chk_dependencies = ctk.CTkCheckBox(t, text="Карта зависимостей", **chk_opts)
        self.chk_dependencies.pack(anchor="w", pady=AppleTheme.SP_6)

        btn_frame = ctk.CTkFrame(t, fg_color=AppleTheme.TRANSPARENT)
        btn_frame.pack(fill="x", pady=(AppleTheme.SP_20, AppleTheme.SP_4))

        self.btn_add_folder = ctk.CTkButton(btn_frame, text="+ Папка", width=130, **btn_opts)
        self.btn_add_folder.pack(side="left")

        self.btn_add_github = ctk.CTkButton(
            btn_frame, text="+ GitHub", width=130,
            fg_color=AppleTheme.SUCCESS, text_color="#ffffff",
            hover_color=AppleTheme.BORDER, corner_radius=AppleTheme.RADIUS_PILL,
            font=AppleTheme.FONT_BODY_SM, height=AppleTheme.HEIGHT_BTN_SEC
        )
        self.btn_add_github.pack(side="right")

        self.btn_scan = ctk.CTkButton(
            t, text="🔍 Сканировать (Preview)",
            fg_color=AppleTheme.AZURE, hover_color=AppleTheme.AZURE_HOVER,
            text_color="#ffffff", font=AppleTheme.FONT_BUTTON,
            corner_radius=AppleTheme.RADIUS_PILL, height=AppleTheme.HEIGHT_BTN_PRIMARY,
            command=self._on_scan_click
        )
        self.btn_scan.pack(fill="x", pady=(AppleTheme.SP_20, AppleTheme.SP_12))

        self.btn_clear = ctk.CTkButton(
            t, text="Очистить",
            fg_color=AppleTheme.TRANSPARENT, hover_color=AppleTheme.FOG,
            text_color=AppleTheme.GRAPHITE, font=AppleTheme.FONT_BODY,
            corner_radius=AppleTheme.RADIUS_PILL, command=self._on_clear_folders
        )
        self.btn_clear.pack(fill="x", pady=AppleTheme.SP_0)

    def _build_prompt_tab(self):
        t_scroll = ctk.CTkScrollableFrame(self.tab_prompt, fg_color=AppleTheme.TRANSPARENT)
        t_scroll.pack(fill="both", expand=True)
        t = t_scroll

        lbl_opts = {"font": AppleTheme.FONT_BODY_SM, "text_color": AppleTheme.GRAPHITE}
        entry_opts = {"font": AppleTheme.FONT_BODY, "corner_radius": AppleTheme.RADIUS_SMALL}

        ctk.CTkLabel(t, text="Выберите пресет промпта:", **lbl_opts).pack(anchor="w",
                                                                          pady=(AppleTheme.SP_4, AppleTheme.SP_2))
        self.cmb_prompt_presets = ctk.CTkComboBox(t, values=list(PROMPT_PRESETS.keys()),
                                                  command=self._on_prompt_preset_change, **entry_opts)
        self.cmb_prompt_presets.pack(fill="x", pady=(AppleTheme.SP_0, AppleTheme.SP_16))

        ctk.CTkLabel(t, text="Системный промпт:", **lbl_opts).pack(anchor="w")
        self.txt_system_prompt = ctk.CTkTextbox(t, **entry_opts)
        self.txt_system_prompt.pack(fill="both", expand=True, pady=AppleTheme.SP_4)
        self.txt_system_prompt.bind("<KeyRelease>", self._on_prompt_type)

        btn_patch = ctk.CTkButton(
            t, text="Применить JSON-патч от LLM",
            fg_color=AppleTheme.FOG, text_color=AppleTheme.INK, hover_color=AppleTheme.BORDER,
            corner_radius=AppleTheme.RADIUS_PILL, font=AppleTheme.FONT_BUTTON, height=AppleTheme.HEIGHT_BTN_PRIMARY,
            command=self._on_apply_patch
        )
        btn_patch.pack(fill="x", pady=(AppleTheme.SP_16, AppleTheme.SP_4))

    def _build_settings_tab(self):
        t_scroll = ctk.CTkScrollableFrame(self.tab_settings, fg_color=AppleTheme.TRANSPARENT)
        t_scroll.pack(fill="both", expand=True)
        t = t_scroll

        chk_opts = {"font": AppleTheme.FONT_BODY, "text_color": AppleTheme.INK}
        lbl_h = {"font": AppleTheme.FONT_BUTTON, "text_color": AppleTheme.INK}

        cli_frame = ctk.CTkFrame(t, fg_color=AppleTheme.TRANSPARENT)
        cli_frame.pack(fill="x", pady=AppleTheme.SP_4)
        ctk.CTkLabel(cli_frame, text="Настройки CLI", **lbl_h).pack(pady=AppleTheme.SP_4)

        self.chk_cli_minify = ctk.CTkCheckBox(cli_frame, text="Minify (сжать)", **chk_opts)
        self.chk_cli_minify.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)
        self.chk_cli_comments = ctk.CTkCheckBox(cli_frame, text="Удалять комментарии", **chk_opts)
        self.chk_cli_comments.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)
        self.chk_cli_secrets = ctk.CTkCheckBox(cli_frame, text="Скрывать секреты", **chk_opts)
        self.chk_cli_secrets.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)
        self.chk_cli_tree = ctk.CTkCheckBox(cli_frame, text="Добавлять дерево файлов", **chk_opts)
        self.chk_cli_tree.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)
        self.chk_cli_skeleton = ctk.CTkCheckBox(cli_frame, text="Skeleton Mode", **chk_opts)
        self.chk_cli_skeleton.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)
        self.chk_cli_gitignore = ctk.CTkCheckBox(cli_frame, text="Учитывать .gitignore", **chk_opts)
        self.chk_cli_gitignore.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)

        ctk.CTkLabel(cli_frame, text="Формат вывода:", font=AppleTheme.FONT_BODY_SM,
                     text_color=AppleTheme.GRAPHITE).pack(anchor="w", padx=AppleTheme.SP_4,
                                                          pady=(AppleTheme.SP_6, AppleTheme.SP_0))
        self.cmb_cli_format = ctk.CTkComboBox(cli_frame, values=["plain", "markdown", "xml"],
                                              corner_radius=AppleTheme.RADIUS_SMALL, font=AppleTheme.FONT_BODY)
        self.cmb_cli_format.pack(fill="x", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)

        theme_frame = ctk.CTkFrame(t, fg_color=AppleTheme.TRANSPARENT)
        theme_frame.pack(fill="x", pady=AppleTheme.SP_12)
        ctk.CTkLabel(theme_frame, text="Внешний вид", **lbl_h).pack(pady=AppleTheme.SP_4)

        self.seg_theme = ctk.CTkSegmentedButton(
            theme_frame, values=["Dark", "Light", "System"],
            command=self._on_theme_change, font=AppleTheme.FONT_BODY,
            selected_color=AppleTheme.FOG, selected_hover_color=AppleTheme.BORDER,
            unselected_color=AppleTheme.CARD,  # ИСПРАВЛЕНО: Вместо TRANSPARENT
            unselected_hover_color=AppleTheme.FOG,
            text_color=AppleTheme.INK
        )
        self.seg_theme.pack(fill="x", padx=AppleTheme.SP_4, pady=AppleTheme.SP_4)
        self.seg_theme.set("System")

        log_frame = ctk.CTkFrame(t, fg_color=AppleTheme.TRANSPARENT)
        log_frame.pack(fill="x", pady=AppleTheme.SP_12)
        ctk.CTkLabel(log_frame, text="Отладка и Логи", **lbl_h).pack(pady=AppleTheme.SP_4)

        self.chk_logging = ctk.CTkCheckBox(log_frame, text="Писать логи (logs/app.log)", **chk_opts)
        self.chk_logging.pack(anchor="w", padx=AppleTheme.SP_4, pady=AppleTheme.SP_6)

    def _on_theme_change(self, value):
        ctk.set_appearance_mode(value)

    def _on_parse_error(self):
        from ..dialogs import InputTextDialog
        d = InputTextDialog(self, "Парсинг лога ошибки", "Вставьте весь лог с ошибками сюда...")
        res = d.get_input()
        if res and res.strip(): self.controller.parse_error_log(res)

    def _on_apply_patch(self):
        from ..dialogs import InputTextDialog
        placeholder = '[\n  {\n    "action": "replace",\n    "file": "main.py",\n    "search": "def test(): pass",\n    "content": "def test(): return True"\n  }\n]'
        d = InputTextDialog(self, "Применить патч LLM", placeholder)
        res = d.get_input()
        if res and res.strip() != placeholder: self.controller.apply_llm_patch(res)

    def _on_save_workspace(self):
        self.on_settings_change()
        path = filedialog.asksaveasfilename(defaultextension=".json", filetypes=[("JSON Workspace", "*.json")])
        if path: self.controller.save_workspace(path)

    def _on_load_workspace(self):
        path = filedialog.askopenfilename(filetypes=[("JSON Workspace", "*.json")])
        if path: self.controller.load_workspace(path)

    def _on_apply_preset(self, choice: str):
        self.controller.apply_preset(choice)

    def _on_prompt_preset_change(self, choice: str):
        prompt_text = PROMPT_PRESETS.get(choice)
        if choice != "Custom" and prompt_text is not None:
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", prompt_text)
            self.on_settings_change()

    def _on_prompt_type(self, _event: Any = None):
        if self.cmb_prompt_presets.get() != "Custom": self.cmb_prompt_presets.set("Custom")

    def _on_scan_click(self):
        self.controller.update_settings(self.get_settings())
        self.controller.scan_only()

    def _on_clear_folders(self):
        self.controller.clear_folders()

    @staticmethod
    def _set_check(chk: Any, val: bool):
        if val:
            chk.select()
        else:
            chk.deselect()

    def update_ui(self, settings: Any):
        self.entry_ext.delete(0, "end");
        self.entry_ext.insert(0, settings.extensions)
        self.entry_ign.delete(0, "end");
        self.entry_ign.insert(0, settings.ignored_paths)
        self._set_check(self.chk_tree, settings.include_tree)
        self._set_check(self.chk_dependencies, settings.include_dependencies)
        self._set_check(self.chk_git, settings.use_git)
        self._set_check(self.chk_gitignore, settings.use_gitignore)
        self._set_check(self.chk_cli_minify, settings.cli_minify)
        self._set_check(self.chk_cli_comments, settings.cli_remove_comments)
        self._set_check(self.chk_cli_secrets, settings.cli_remove_secrets)
        self._set_check(self.chk_cli_tree, settings.cli_include_tree)
        self._set_check(self.chk_cli_skeleton, settings.cli_skeleton_mode)
        self._set_check(self.chk_cli_gitignore, settings.cli_use_gitignore)
        self._set_check(self.chk_logging, getattr(settings, 'enable_logging', True))
        self.cmb_cli_format.set(settings.cli_format)

        current_prompt = self.txt_system_prompt.get("1.0", "end-1c")
        if current_prompt != settings.system_prompt and not self.txt_system_prompt.focus_get():
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", settings.system_prompt)

        found = False
        for name, text in PROMPT_PRESETS.items():
            if name != "Custom" and text.strip() == settings.system_prompt.strip():
                self.cmb_prompt_presets.set(name)
                found = True;
                break
        if not found: self.cmb_prompt_presets.set("Custom")

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
            'cli_format': self.cmb_cli_format.get(),
            'enable_logging': bool(getattr(self, 'chk_logging', ctk.CTkCheckBox(self)).get())
        }

    def set_loading(self, is_loading: bool):
        state = "disabled" if is_loading else "normal"
        self.tab_view.configure(state=state)