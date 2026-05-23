import customtkinter as ctk
from tkinter import filedialog
import os


class ActionPanel(ctk.CTkFrame):
    def __init__(self, parent, on_run_callback):
        super().__init__(parent, fg_color="transparent")
        self.on_run = on_run_callback
        self.selected_template_path = ""
        self._init_options()
        self._init_buttons()

    def _init_options(self):
        self.opts_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.opts_frame.pack(fill="x", pady=(0, 10))

        self.left_opts = ctk.CTkFrame(self.opts_frame, fg_color="transparent")
        self.left_opts.pack(side="left")
        self.chk_minify = ctk.CTkCheckBox(self.left_opts, text="Minify")
        self.chk_minify.pack(side="left", padx=5)
        self.chk_comments = ctk.CTkCheckBox(self.left_opts, text="No Comments")
        self.chk_comments.pack(side="left", padx=5)
        self.chk_secrets = ctk.CTkCheckBox(self.left_opts, text="No Secrets")
        self.chk_secrets.pack(side="left", padx=5)
        self.chk_skeleton = ctk.CTkCheckBox(self.left_opts, text="Skeleton ☠️")
        self.chk_skeleton.pack(side="left", padx=5)

        self.right_opts = ctk.CTkFrame(self.opts_frame, fg_color="transparent")
        self.right_opts.pack(side="right")
        self.seg_format = ctk.CTkSegmentedButton(
            self.right_opts,
            values=["markdown", "xml", "plain", "custom"],
            command=self._on_format_change
        )
        self.seg_format.pack(side="left")
        self.seg_format.set("markdown")

        self.btn_template = ctk.CTkButton(
            self.right_opts, text="📂", width=30, command=self._choose_template, fg_color="gray"
        )
        self.lbl_template = ctk.CTkLabel(self.right_opts, text="", text_color="gray")

    def _init_buttons(self):
        self.btns_frame = ctk.CTkFrame(self)
        self.btns_frame.pack(fill="x", pady=(0, 10))

        # Добавлена кнопка превью
        ctk.CTkButton(self.btns_frame, text="👀 Предпросмотр",
                      command=lambda: self.on_run("preview")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

        ctk.CTkButton(self.btns_frame, text="📋 В Буфер",
                      command=lambda: self.on_run("clipboard")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

        ctk.CTkButton(self.btns_frame, text="💾 В Файл",
                      command=lambda: self.on_run("file")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

        ctk.CTkButton(self.btns_frame, text="📄 В PDF",
                      command=lambda: self.on_run("pdf")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

    def _on_format_change(self, value):
        if value == "custom":
            self.btn_template.pack(side="left", padx=(5, 0))
            self.lbl_template.pack(side="left", padx=(5, 0))
            if not self.selected_template_path:
                self.lbl_template.configure(text="(не выбран)")
        else:
            self.btn_template.pack_forget()
            self.lbl_template.pack_forget()

    def _choose_template(self):
        file_path = filedialog.askopenfilename(
            title="Выберите шаблон Jinja2",
            filetypes=[("Jinja2 Template", "*.jinja2"), ("Text File", "*.txt"), ("All Files", "*.*")]
        )
        if file_path:
            self.selected_template_path = file_path
            self.lbl_template.configure(text=os.path.basename(file_path))

    def update_ui(self, settings):
        self._set_check(self.chk_minify, settings.minify)
        self._set_check(self.chk_comments, settings.remove_comments)
        self._set_check(self.chk_secrets, settings.remove_secrets)
        self._set_check(self.chk_skeleton, settings.skeleton_mode)
        self.seg_format.set(settings.output_format)

        if settings.output_format == "custom":
            self.selected_template_path = settings.template_path
            self._on_format_change("custom")
            if self.selected_template_path:
                self.lbl_template.configure(text=os.path.basename(self.selected_template_path))
        else:
            self._on_format_change(settings.output_format)

    def get_settings(self):
        return {
            'minify': bool(self.chk_minify.get()),
            'remove_comments': bool(self.chk_comments.get()),
            'remove_secrets': bool(self.chk_secrets.get()),
            'skeleton_mode': bool(self.chk_skeleton.get()),
            'output_format': self.seg_format.get(),
            'template_path': self.selected_template_path
        }

    @staticmethod
    def _set_check(chk, val):
        if val:
            chk.select()
        else:
            chk.deselect()