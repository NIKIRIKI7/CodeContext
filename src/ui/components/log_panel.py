import customtkinter as ctk
from ..theme import AppleTheme

class LogPanel(ctk.CTkTextbox):
    def __init__(self, parent):
        super().__init__(
            parent,
            font=AppleTheme.FONT_CODE_SM,
            fg_color=AppleTheme.CARD,
            text_color=AppleTheme.INK,
            corner_radius=AppleTheme.RADIUS_CARD
        )
        self.configure(state="disabled")

    def update_logs(self, logs):
        self.configure(state="normal")
        self.delete("1.0", "end")
        for log in logs:
            self.insert("end", f"{log}\n")
        self.see("end")
        self.configure(state="disabled")