import customtkinter as ctk

class LogPanel(ctk.CTkTextbox):
    def __init__(self, parent):
        super().__init__(parent, font=("Consolas", 12))
        self.configure(state="disabled")

    def update_logs(self, logs):
        """Обновляет логи. Можно оптимизировать, сравнивая длину."""
        self.configure(state="normal")
        self.delete("1.0", "end")
        for log in logs:
            self.insert("end", f"{log}\n")
        self.see("end")
        self.configure(state="disabled")