import customtkinter as ctk


class FolderList(ctk.CTkScrollableFrame):
    def __init__(self, parent, on_edit_callback, on_delete_callback):
        super().__init__(parent, height=120, label_text="Источники (Перетащите папки сюда)")
        self.on_edit = on_edit_callback
        self.on_delete = on_delete_callback
        self._last_hash = None

    def update_ui(self, selected_folders, temp_folders):
        current_hash = tuple(selected_folders)
        if self._last_hash == current_hash:
            return
        self._last_hash = current_hash

        for widget in self.winfo_children():
            widget.destroy()

        # --- Empty State ---
        if not selected_folders:
            lbl = ctk.CTkLabel(self, text="Источники не выбраны. Перетащите папки сюда.", text_color="gray")
            lbl.pack(expand=True, fill="both", pady=20)
            return
        # -------------------

        for folder in selected_folders:
            row = ctk.CTkFrame(self, fg_color="transparent")
            row.pack(fill="x", pady=2)

            is_temp = folder in temp_folders
            prefix = "☁️" if is_temp else "📂"

            label = ctk.CTkLabel(row, text=f"{prefix} {folder}", anchor="w")
            label.pack(side="left", padx=5, expand=True, fill="x")

            btn_edit = ctk.CTkButton(row, text="✏️", width=30, height=24,
                                     fg_color="transparent", border_width=1,
                                     text_color=("gray10", "gray90"),
                                     command=lambda p=folder: self.on_edit(p))
            btn_edit.pack(side="right", padx=2)

            btn_del = ctk.CTkButton(row, text="❌", width=30, height=24,
                                    fg_color="transparent", border_width=1,
                                    hover_color="#c85a5a",
                                    command=lambda p=folder: self.on_delete(p))
            btn_del.pack(side="right", padx=2)