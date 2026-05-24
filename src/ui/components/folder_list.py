import customtkinter as ctk
from ..theme import AppleTheme


class FolderList(ctk.CTkScrollableFrame):
    def __init__(self, parent, on_edit_callback, on_delete_callback):
        super().__init__(
            parent,
            height=AppleTheme.HEIGHT_LOGS,
            fg_color=AppleTheme.CARD,
            corner_radius=AppleTheme.RADIUS_CARD
        )
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

        if not selected_folders:
            lbl = ctk.CTkLabel(
                self,
                text="Источники не выбраны. Перетащите папки сюда.",
                font=AppleTheme.FONT_BODY,
                text_color=AppleTheme.GRAPHITE
            )
            lbl.pack(expand=True, fill="both", pady=AppleTheme.SP_20)
            return

        for folder in selected_folders:
            row = ctk.CTkFrame(self, fg_color=AppleTheme.TRANSPARENT)
            row.pack(fill="x", pady=AppleTheme.SP_4, padx=AppleTheme.SP_12)

            is_temp = folder in temp_folders
            prefix = "☁️" if is_temp else "📂"

            label = ctk.CTkLabel(
                row,
                text=f"{prefix} {folder}",
                font=AppleTheme.FONT_BODY,
                text_color=AppleTheme.INK,
                anchor="w"
            )
            label.pack(side="left", padx=AppleTheme.SP_4, expand=True, fill="x")

            btn_opts = {
                "width": AppleTheme.SP_32,
                "height": AppleTheme.SP_32,
                "fg_color": AppleTheme.FOG,
                "hover_color": AppleTheme.BORDER,
                "text_color": AppleTheme.INK,
                "corner_radius": AppleTheme.RADIUS_SMALL
            }

            btn_edit = ctk.CTkButton(row, text="✏️", command=lambda p=folder: self.on_edit(p), **btn_opts)
            btn_edit.pack(side="right", padx=AppleTheme.SP_4)

            btn_del = ctk.CTkButton(row, text="✕", command=lambda p=folder: self.on_delete(p), **btn_opts)
            btn_del.pack(side="right", padx=AppleTheme.SP_4)