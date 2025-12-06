import customtkinter as ctk


class ActionPanel(ctk.CTkFrame):
    def __init__(self, parent, on_run_callback):
        super().__init__(parent, fg_color="transparent")
        self.on_run = on_run_callback

        self._init_options()
        self._init_buttons()

    def _init_options(self):
        self.opts_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.opts_frame.pack(fill="x", pady=(0, 10))

        self.chk_minify = ctk.CTkCheckBox(self.opts_frame, text="Minify")
        self.chk_minify.pack(side="left", padx=10)

        self.chk_comments = ctk.CTkCheckBox(self.opts_frame, text="No Comments")
        self.chk_comments.pack(side="left", padx=10)

        self.chk_secrets = ctk.CTkCheckBox(self.opts_frame, text="No Secrets")
        self.chk_secrets.pack(side="left", padx=10)

        self.chk_skeleton = ctk.CTkCheckBox(self.opts_frame, text="Skeleton ☠️")
        self.chk_skeleton.pack(side="left", padx=10)

        self.seg_format = ctk.CTkSegmentedButton(self.opts_frame, values=["markdown", "xml", "plain"])
        self.seg_format.pack(side="right")
        self.seg_format.set("markdown")

    def _init_buttons(self):
        self.btns_frame = ctk.CTkFrame(self)
        self.btns_frame.pack(fill="x", pady=(0, 10))

        ctk.CTkButton(self.btns_frame, text="В Буфер",
                      command=lambda: self.on_run("clipboard")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

        ctk.CTkButton(self.btns_frame, text="В Файл",
                      command=lambda: self.on_run("file")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

        ctk.CTkButton(self.btns_frame, text="В PDF",
                      command=lambda: self.on_run("pdf")).pack(side="left", expand=True, fill="x", padx=5, pady=5)

    def update_ui(self, settings):
        self._set_check(self.chk_minify, settings.minify)
        self._set_check(self.chk_comments, settings.remove_comments)
        self._set_check(self.chk_secrets, settings.remove_secrets)
        self._set_check(self.chk_skeleton, settings.skeleton_mode)
        # Segmented button обычно не сбрасывается из настроек в коде main_window,
        # но добавим для полноты, если это есть в Settings
        self.seg_format.set(settings.output_format)

    def get_settings(self):
        return {
            'minify': bool(self.chk_minify.get()),
            'remove_comments': bool(self.chk_comments.get()),
            'remove_secrets': bool(self.chk_secrets.get()),
            'skeleton_mode': bool(self.chk_skeleton.get()),
            'output_format': self.seg_format.get()
        }

    def _set_check(self, chk, val):
        if val:
            chk.select()
        else:
            chk.deselect()