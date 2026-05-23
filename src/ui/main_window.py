import os
import customtkinter as ctk
from tkinter import filedialog, messagebox
from tkinterdnd2 import TkinterDnD, DND_FILES  # type: ignore

from ..store.store import Store
from ..controllers.main_controller import MainController
from .dialogs import EditFolderDialog, PreviewDialog  # <-- Added import
from .components.sidebar import Sidebar
from .components.folder_list import FolderList
from .components.action_panel import ActionPanel
from .components.log_panel import LogPanel
from .components.status_bar import StatusBar
from .components.file_tree import FileTree

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")


class MainWindow(ctk.CTk, TkinterDnD.DnDWrapper):  # type: ignore
    def __init__(self, store: Store, controller: MainController):
        super().__init__()
        self.TkdndVersion = getattr(TkinterDnD, "_require")(self)
        self.store = store
        self.controller = controller

        self.title("CodeContext AI - Modular Architecture")
        self.geometry("1150x850")

        self.unsubscribe = self.store.subscribe(self._on_store_changed_threadsafe)

        self._preview_dialog = None  # <-- For tracking the preview window

        self._init_ui()
        self._bind_hotkeys()  # <-- Hotkeys

        self.drop_target_register(DND_FILES)
        self.dnd_bind('<<Drop>>', self._on_drop)
        self.controller.load_initial_settings()

    def _bind_hotkeys(self):
        # Quick build to clipboard: Ctrl + Enter
        self.bind("<Control-Return>", lambda e: self._on_run("clipboard"))
        # Quick clear: Esc
        self.bind("<Escape>", lambda e: self.controller.clear_folders())

    def _init_ui(self):
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        self.sidebar = Sidebar(self, self.controller, self._on_ui_settings_change)
        self.sidebar.grid(row=0, column=0, sticky="nsew")

        self.sidebar.btn_add_folder.configure(command=self._on_add_folder)
        self.sidebar.btn_add_github.configure(command=self._on_add_github)
        self.sidebar.btn_clear.configure(command=self._on_clear_folders)
        self.sidebar.btn_save.configure(command=self._on_save_settings)
        self.sidebar.btn_reset.configure(command=self._on_reset_settings)
        self.sidebar.btn_install_ctx.configure(command=self._on_install_context)
        self.sidebar.btn_remove_ctx.configure(command=self._on_remove_context)

        main_frame = ctk.CTkFrame(self, fg_color="transparent")
        main_frame.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)
        main_frame.grid_columnconfigure(0, weight=1)
        main_frame.grid_rowconfigure(0, weight=0)
        main_frame.grid_rowconfigure(1, weight=1)
        main_frame.grid_rowconfigure(2, weight=0)
        main_frame.grid_rowconfigure(3, weight=0)
        main_frame.grid_rowconfigure(4, weight=0)

        self.folder_list = FolderList(main_frame, self._on_edit_folder, self._on_remove_folder)
        self.folder_list.grid(row=0, column=0, sticky="ew", pady=(0, 10))

        self.file_tree = FileTree(main_frame, self._on_tree_toggle, self._on_tree_context_action)
        self.file_tree.grid(row=1, column=0, sticky="nsew", pady=(0, 10))

        self.action_panel = ActionPanel(main_frame, self._on_run)
        self.action_panel.grid(row=2, column=0, sticky="ew", pady=(0, 10))

        self.log_panel = LogPanel(main_frame)
        self.log_panel.configure(height=100)
        self.log_panel.grid(row=3, column=0, sticky="ew")

        self.status_bar = StatusBar(main_frame)
        self.status_bar.grid(row=4, column=0, sticky="ew", pady=(10, 0))

    def _on_store_changed_threadsafe(self, state):
        self.after(0, self._on_store_changed, state)

    def _on_store_changed(self, state):
        self.sidebar.update_ui(state.settings)
        self.sidebar.set_loading(state.is_loading)
        self.folder_list.update_ui(state.selected_folders, state.temp_folders)

        current_tree_files = self.file_tree.file_paths
        if state.scanned_files_paths and state.scanned_files_paths != current_tree_files:
            self.file_tree.populate(state.scanned_files_paths)
        elif not state.scanned_files_paths and current_tree_files:
            self.file_tree.delete_all()

        self.action_panel.update_ui(state.settings)
        self.log_panel.update_logs(state.logs)
        self.status_bar.update_ui(state.status_message, state.progress, state.total_tokens)

        # --- UI Logic for Preview ---
        if state.show_preview and not self._preview_dialog:
            self._show_preview_dialog(state.preview_text)
        elif not state.show_preview and self._preview_dialog:
            self._preview_dialog.destroy()
            self._preview_dialog = None

    def _show_preview_dialog(self, text: str):
        self._preview_dialog = PreviewDialog(self, text, self.controller.close_preview)
        self._preview_dialog.grab_set()

    def _on_tree_toggle(self, path, state):
        self.controller.toggle_file_exclusion(path, state)

    def _on_tree_context_action(self, path: str, is_deep: bool):
        self.controller.copy_file_with_dependencies(path, is_deep)

    def _on_drop(self, event):
        if not event.data: return
        try:
            raw_data = event.data
            paths = self.tk.splitlist(raw_data)
            count = 0
            for path in paths:
                path = path.strip('{}')
                if os.path.isdir(path):
                    self.controller.add_folder(path)
                    count += 1
                elif os.path.isfile(path):
                    folder = os.path.dirname(path)
                    self.controller.add_folder(folder)
                    count += 1
        except Exception as e:
            print(f"Drop error: {e}")

    def _on_ui_settings_change(self):
        data = self._collect_settings()
        self.controller.update_settings(data)

    def _collect_settings(self):
        s_data = self.sidebar.get_settings()
        a_data = self.action_panel.get_settings()
        return {**s_data, **a_data}

    def _on_add_folder(self):
        path = filedialog.askdirectory()
        if path:
            self.controller.add_folder(path)

    def _on_add_github(self):
        dialog = ctk.CTkInputDialog(text="Введите URL репозитория:", title="GitHub Clone")
        url = dialog.get_input()
        if url:
            self.controller.add_github_repo(url)

    def _on_remove_folder(self, path):
        self.controller.remove_folder(path)

    def _on_edit_folder(self, path):
        dialog = EditFolderDialog(self, path)
        new_path = dialog.get_input()
        if new_path:
            self.controller.edit_folder(path, new_path)

    def _on_clear_folders(self):
        self.controller.clear_folders()

    def _on_save_settings(self):
        data = self._collect_settings()
        self.controller.update_settings(data)
        self.controller.save_settings()
        messagebox.showinfo("Сохранено", "Настройки успешно сохранены!")

    def _on_reset_settings(self):
        if messagebox.askyesno("Сброс", "Сбросить все настройки?"):
            self.controller.reset_settings()

    def _on_install_context(self):
        try:
            success, msg = self.controller.install_context_menu()
            if success:
                messagebox.showinfo("Успех", msg)
            else:
                if "Запрошены права" in msg:
                    messagebox.showinfo("Инфо", msg)
                else:
                    messagebox.showerror("Ошибка", msg)
        except SystemExit:
            pass

    def _on_remove_context(self):
        try:
            success, msg = self.controller.remove_context_menu()
            if success:
                messagebox.showinfo("Успех", msg)
            else:
                if "Запрошены права" in msg:
                    messagebox.showinfo("Инфо", msg)
                else:
                    messagebox.showerror("Ошибка", msg)
        except SystemExit:
            pass

    def _on_run(self, target):
        data = self._collect_settings()
        self.controller.update_settings(data)
        state = self.store.state
        save_path = None

        if target == 'file':
            if state.settings.output_format == 'markdown':
                ext = ".md"
            elif state.settings.output_format == 'xml':
                ext = ".xml"
            else:
                ext = ".txt"
            save_path = filedialog.asksaveasfilename(defaultextension=ext)
            if not save_path: return
        elif target == 'pdf':
            save_path = filedialog.asksaveasfilename(defaultextension=".pdf")
            if not save_path: return

        success, msg = self.controller.start_processing(target, save_path)
        if not success:
            messagebox.showwarning("Внимание", msg)

    def on_closing(self):
        self.controller.clear_folders()
        if self.unsubscribe:
            self.unsubscribe()
        self.destroy()