import customtkinter as ctk
from ..theme import AppleTheme


class StatusBar(ctk.CTkFrame):
    def __init__(self, parent):
        super().__init__(
            parent,
            height=AppleTheme.SP_32,
            fg_color=AppleTheme.TRANSPARENT
        )

        self.progress_bar = ctk.CTkProgressBar(
            self,
            height=AppleTheme.SP_4,
            progress_color=AppleTheme.AZURE[0],
            fg_color=AppleTheme.BORDER[0]
        )
        self.progress_bar.pack(fill="x", side="top", pady=(AppleTheme.SP_0, AppleTheme.SP_12))
        self.progress_bar.set(0)

        self.lbl_status = ctk.CTkLabel(
            self,
            text="Idle",
            font=AppleTheme.FONT_BODY,
            text_color=AppleTheme.GRAPHITE
        )
        self.lbl_status.pack(side="left", padx=AppleTheme.SP_12)

        self.lbl_tokens = ctk.CTkLabel(
            self,
            text="Tokens: 0",
            font=AppleTheme.FONT_BODY,
            text_color=AppleTheme.GRAPHITE
        )
        self.lbl_tokens.pack(side="right", padx=AppleTheme.SP_12)

    def update_ui(self, status_message, progress, tokens):
        self.lbl_status.configure(text=status_message)
        self.progress_bar.set(progress)
        self.lbl_tokens.configure(text=f"Tokens: {tokens}")