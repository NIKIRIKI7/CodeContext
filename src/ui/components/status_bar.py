import customtkinter as ctk


class StatusBar(ctk.CTkFrame):
    def __init__(self, parent):
        super().__init__(parent, height=30)

        self.progress_bar = ctk.CTkProgressBar(self)
        self.progress_bar.pack(fill="x", side="top")
        self.progress_bar.set(0)

        self.lbl_status = ctk.CTkLabel(self, text="Idle", text_color="gray")
        self.lbl_status.pack(side="left", padx=5)

        self.lbl_tokens = ctk.CTkLabel(self, text="Tokens: 0", text_color="gray")
        self.lbl_tokens.pack(side="right", padx=5)

    def update_ui(self, status_message, progress, tokens):
        self.lbl_status.configure(text=status_message)
        self.progress_bar.set(progress)
        self.lbl_tokens.configure(text=f"Tokens: {tokens}")