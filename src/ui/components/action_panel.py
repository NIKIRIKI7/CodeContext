import customtkinter as ctk
from ..theme import AppleTheme


class ActionPanel(ctk.CTkFrame):
    def __init__(self, parent, on_run_callback):
        super().__init__(
            parent,
            fg_color=AppleTheme.CARD,
            corner_radius=AppleTheme.RADIUS_CARD
        )
        self.on_run = on_run_callback
        self.selected_template_path = ""

        self.container = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
        self.container.pack(fill="both", expand=True, padx=AppleTheme.SP_28, pady=AppleTheme.SP_20)

        self._init_options()
        self._init_buttons()

    def _init_options(self):
        self.opts_frame = ctk.CTkFrame(self.container, fg_color=AppleTheme.TRANSPARENT)
        self.opts_frame.pack(fill="x", pady=(AppleTheme.SP_0, AppleTheme.SP_16))

        self.left_opts = ctk.CTkFrame(self.opts_frame, fg_color=AppleTheme.TRANSPARENT)
        self.left_opts.pack(side="left")

        chk_opts = {
            "font": AppleTheme.FONT_BODY,
            "text_color": AppleTheme.INK
        }

        self.chk_minify = ctk.CTkCheckBox(self.left_opts, text="Minify", **chk_opts)
        self.chk_minify.pack(side="left", padx=(AppleTheme.SP_0, AppleTheme.SP_16))

        self.chk_comments = ctk.CTkCheckBox(self.left_opts, text="No Comments", **chk_opts)
        self.chk_comments.pack(side="left", padx=AppleTheme.SP_16)

        self.chk_secrets = ctk.CTkCheckBox(self.left_opts, text="No Secrets", **chk_opts)
        self.chk_secrets.pack(side="left", padx=AppleTheme.SP_16)

        self.chk_skeleton = ctk.CTkCheckBox(self.left_opts, text="Skeleton ☠️", **chk_opts)
        self.chk_skeleton.pack(side="left", padx=AppleTheme.SP_16)

        self.right_opts = ctk.CTkFrame(self.opts_frame, fg_color=AppleTheme.TRANSPARENT)
        self.right_opts.pack(side="right")

        self.seg_format = ctk.CTkSegmentedButton(
            self.right_opts,
            values=["markdown", "xml", "plain", "custom"],
            command=self._on_format_change,
            font=AppleTheme.FONT_BODY,
            selected_color=AppleTheme.FOG,
            selected_hover_color=AppleTheme.BORDER,
            unselected_color=AppleTheme.CARD,  # ИСПРАВЛЕНО: Вместо TRANSPARENT
            unselected_hover_color=AppleTheme.FOG,
            text_color=AppleTheme.INK
        )
        self.seg_format.pack(side="left")
        self.seg_format.set("markdown")

    def _init_buttons(self):
        self.btns_frame = ctk.CTkFrame(self.container, fg_color=AppleTheme.TRANSPARENT)
        self.btns_frame.pack(fill="x")

        ghost_opts = {
            "fg_color": AppleTheme.FOG,
            "text_color": AppleTheme.INK,
            "hover_color": AppleTheme.BORDER,
            "corner_radius": AppleTheme.RADIUS_PILL,
            "font": AppleTheme.FONT_BUTTON,
            "height": AppleTheme.HEIGHT_BTN_PRIMARY
        }

        cta_opts = {
            "fg_color": AppleTheme.AZURE,
            "text_color": "#ffffff",
            "hover_color": AppleTheme.AZURE_HOVER,
            "corner_radius": AppleTheme.RADIUS_PILL,
            "font": AppleTheme.FONT_BUTTON,
            "height": AppleTheme.HEIGHT_BTN_PRIMARY
        }

        btn_preview = ctk.CTkButton(self.btns_frame, text="👀 Предпросмотр", command=lambda: self.on_run("preview"),
                                    **ghost_opts)
        btn_preview.pack(side="left", expand=True, fill="x", padx=AppleTheme.SP_6)

        btn_copy = ctk.CTkButton(self.btns_frame, text="📋 В Буфер обмена", command=lambda: self.on_run("clipboard"),
                                 **cta_opts)
        btn_copy.pack(side="left", expand=True, fill="x", padx=AppleTheme.SP_6)

        btn_file = ctk.CTkButton(self.btns_frame, text="💾 Сохранить в Файл", command=lambda: self.on_run("file"),
                                 **ghost_opts)
        btn_file.pack(side="left", expand=True, fill="x", padx=AppleTheme.SP_6)

    def _on_format_change(self, value):
        pass

    def update_ui(self, settings):
        self._set_check(self.chk_minify, settings.minify)
        self._set_check(self.chk_comments, settings.remove_comments)
        self._set_check(self.chk_secrets, settings.remove_secrets)
        self._set_check(self.chk_skeleton, settings.skeleton_mode)
        self.seg_format.set(settings.output_format)

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