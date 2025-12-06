import os
import customtkinter as ctk
from tkinter import filedialog, messagebox
from tkinterdnd2 import TkinterDnD, DND_FILES
from ..store.store import Store
from ..controllers.main_controller import MainController
from .dialogs import EditFolderDialog
from .components.sidebar import Sidebar
from .components.folder_list import FolderList
from .components.action_panel import ActionPanel
from .components.log_panel import LogPanel
from .components.status_bar import StatusBar
from .components.file_tree import FileTree

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")


class MainWindow(ctk.CTk, TkinterDnD.DnDWrapper):
    def __init__(self, store: Store, controller: MainController):
        super().__init__()
        self.TkdndVersion = TkinterDnD._require(self)
        self.store = store
        self.controller = controller

        self.title("CodeContext AI - Modular Architecture")
        self.geometry("1150x850")

        # Подписка на изменения Store с потокобезопасной оберткой
        self.unsubscribe = self.store.subscribe(self._on_store_changed_threadsafe)

        self._init_ui()

        # Настройка Drag & Drop
        self.drop_target_register(DND_FILES)
        self.dnd_bind('<<Drop>>', self._on_drop)

        # Загрузка начальных настроек
        self.controller.load_initial_settings()

    def _init_ui(self):
        """Инициализация компонентов интерфейса"""
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # 1. Сайдбар (Левая панель)
        self.sidebar = Sidebar(self, self.controller, self._on_ui_settings_change)
        self.sidebar.grid(row=0, column=0, sticky="nsew")

        # Привязка команд к кнопкам сайдбара
        self.sidebar.btn_add_folder.configure(command=self._on_add_folder)
        self.sidebar.btn_add_github.configure(command=self._on_add_github)
        self.sidebar.btn_clear.configure(command=self._on_clear_folders)
        self.sidebar.btn_save.configure(command=self._on_save_settings)
        self.sidebar.btn_reset.configure(command=self._on_reset_settings)
        self.sidebar.btn_install_ctx.configure(command=self._on_install_context)
        self.sidebar.btn_remove_ctx.configure(command=self._on_remove_context)

        # 2. Основная панель (Правая часть)
        main_frame = ctk.CTkFrame(self, fg_color="transparent")
        main_frame.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)

        # Настройка сетки внутри main_frame
        main_frame.grid_columnconfigure(0, weight=1)

        main_frame.grid_rowconfigure(0, weight=0)  # Folder List (Фикс. высота)
        main_frame.grid_rowconfigure(1, weight=1)  # FILE TREE (Растягивается)
        main_frame.grid_rowconfigure(2, weight=0)  # Action Panel
        main_frame.grid_rowconfigure(3, weight=0)  # Log Panel
        main_frame.grid_rowconfigure(4, weight=0)  # Status Bar

        # 2.1 Список папок-источников
        self.folder_list = FolderList(main_frame, self._on_edit_folder, self._on_remove_folder)
        self.folder_list.grid(row=0, column=0, sticky="ew", pady=(0, 10))

        # 2.2 Дерево файлов (Новый компонент)
        self.file_tree = FileTree(main_frame, self._on_tree_toggle)
        self.file_tree.grid(row=1, column=0, sticky="nsew", pady=(0, 10))

        # 2.3 Панель действий (Кнопки запуска и опции)
        self.action_panel = ActionPanel(main_frame, self._on_run)
        self.action_panel.grid(row=2, column=0, sticky="ew", pady=(0, 10))

        # 2.4 Лог панель
        self.log_panel = LogPanel(main_frame)
        self.log_panel.configure(height=100)  # Ограничиваем высоту логов, чтобы дерево было больше
        self.log_panel.grid(row=3, column=0, sticky="ew")

        # 2.5 Статус бар
        self.status_bar = StatusBar(main_frame)
        self.status_bar.grid(row=4, column=0, sticky="ew", pady=(10, 0))

    def _on_store_changed_threadsafe(self, state):
        """Вызов обновления UI из главного потока Tkinter"""
        self.after(0, lambda: self._on_store_changed(state))

    def _on_store_changed(self, state):
        """Реакция на изменение стейта"""
        self.sidebar.update_ui(state.settings)
        self.sidebar.set_loading(state.is_loading)

        self.folder_list.update_ui(state.selected_folders, state.temp_folders)

        # Логика обновления дерева: обновляем только если список отсканированных файлов изменился
        current_tree_files = self.file_tree.file_paths

        # Если пришли новые файлы (scan success)
        if state.scanned_files_paths and state.scanned_files_paths != current_tree_files:
            self.file_tree.populate(state.scanned_files_paths)

        # Если файлы были очищены (clear folders)
        elif not state.scanned_files_paths and current_tree_files:
            self.file_tree.delete_all()

        self.action_panel.update_ui(state.settings)
        self.log_panel.update_logs(state.logs)
        self.status_bar.update_ui(state.status_message, state.progress, state.total_tokens)

    def _on_tree_toggle(self, path, state):
        """Коллбек при клике на чекбокс в дереве"""
        self.controller.toggle_file_exclusion(path, state)

    def _on_drop(self, event):
        """Обработка Drag & Drop"""
        if not event.data: return
        try:
            raw_data = event.data
            # TkinterDnD возвращает пути в фигурных скобках, если есть пробелы
            paths = self.tk.splitlist(raw_data)
            count = 0
            for path in paths:
                path = path.strip('{}')
                if os.path.isdir(path):
                    self.controller.add_folder(path)
                    count += 1
                elif os.path.isfile(path):
                    # Если перетащили файл, добавляем его папку
                    folder = os.path.dirname(path)
                    self.controller.add_folder(folder)
                    count += 1
        except Exception as e:
            print(f"Drop error: {e}")

    def _on_ui_settings_change(self):
        """Вызывается компонентами, если нужно срочно обновить состояние (например, при вводе)"""
        data = self._collect_settings()
        self.controller.update_settings(data)

    def _collect_settings(self):
        """Собирает настройки со всех компонентов"""
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
        """Запуск обработки (кнопки В Буфер / В Файл / PDF)"""
        data = self._collect_settings()
        self.controller.update_settings(data)

        state = self.store.state
        save_path = None

        if target == 'file':
            # Определяем расширение по формату
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
        """Очистка при закрытии"""
        self.controller.clear_folders()
        if self.unsubscribe:
            self.unsubscribe()
        self.destroy()