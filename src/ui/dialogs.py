import customtkinter as ctk
from tkinter import filedialog


class EditFolderDialog(ctk.CTkToplevel):
    def __init__(self, parent, initial_path: str):
        super().__init__(parent)
        self.title("Редактирование")
        self.geometry("500x160")
        self.resizable(False, False)
        self.result = None

        # Делаем окно модальным
        self.transient(parent)
        self.grab_set()

        # Центрирование относительно родителя
        self.update_idletasks()
        try:
            x = parent.winfo_x() + (parent.winfo_width() // 2) - (500 // 2)
            y = parent.winfo_y() + (parent.winfo_height() // 2) - (160 // 2)
            self.geometry(f"+{x}+{y}")
        except:
            pass

        ctk.CTkLabel(self, text="Измените путь:", font=("Arial", 14)).pack(pady=(20, 5))

        self.input_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.input_frame.pack(fill="x", padx=20, pady=10)

        self.entry = ctk.CTkEntry(self.input_frame)
        self.entry.pack(side="left", fill="x", expand=True, padx=(0, 10))
        self.entry.insert(0, initial_path)

        self.btn_browse = ctk.CTkButton(self.input_frame, text="📁", width=40, command=self._on_browse)
        self.btn_browse.pack(side="right")

        self.btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.btn_frame.pack(fill="x", padx=20, pady=5)

        self.btn_ok = ctk.CTkButton(self.btn_frame, text="OK", width=100, command=self._on_ok)
        self.btn_ok.pack(side="left", expand=True)

        self.btn_cancel = ctk.CTkButton(self.btn_frame, text="Отмена", width=100, fg_color="transparent",
                                        border_width=1, command=self.destroy)
        self.btn_cancel.pack(side="right", expand=True)

        self.protocol("WM_DELETE_WINDOW", self.destroy)
        self.entry.focus_set()
        self.wait_window()

    def _on_browse(self):
        path = filedialog.askdirectory()
        if path:
            self.entry.delete(0, 'end')
            self.entry.insert(0, path.replace('/', '\\'))

    def _on_ok(self):
        self.result = self.entry.get()
        self.destroy()

    def get_input(self):
        return self.result