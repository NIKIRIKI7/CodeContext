import threading
import customtkinter as ctk
from tkinter import filedialog, messagebox
import sys
import os

from ..store.store import Store
from ..store.state import ProcessedFile  # Важно: импортируем DTO
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import *
from ..utils.config import PRESETS, DEFAULT_SYSTEM_PROMPT

# Импорт независимых сервисов
from ..services.file_service import FileService
from ..services.processing_service import ProcessingService
from ..services.cleaner_service import CleanerService
from ..services.token_service import TokenService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..services.integration_service import IntegrationService

# Data Layer
from ..data.file_system_repository import FileSystemRepository
from ..data.settings_repository import SettingsRepository

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")


class MainWindow(ctk.CTk):
    """Главное окно приложения (View & Controller)"""

    def __init__(self, store: Store, dispatcher: Dispatcher):
        super().__init__()
        self.store = store
        self.dispatcher = dispatcher

        # --- Dependency Injection & Initialization ---

        # 1. Data Layer
        fs_repo = FileSystemRepository()
        self.settings_repo = SettingsRepository()

        # 2. Service Layer (Все сервисы независимы!)
        self.file_service = FileService(fs_repo)
        self.process_service = ProcessingService(fs_repo)  # Только чтение
        self.cleaner_service = CleanerService()  # Только очистка
        self.token_service = TokenService()  # Только подсчет
        self.format_service = FormattingService()
        self.output_service = OutputService()
        self.integration_service = IntegrationService()

        # Настройка окна
        self.title("CodeContext AI - Clean Architecture")
        self.geometry("1150x850")

        # Подписка на Store
        self.unsubscribe = self.store.subscribe(self._on_store_changed)

        self._init_ui()
        self._load_initial_settings()

    def _init_ui(self):
        """Создание UI элементов"""
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # === Sidebar ===
        sidebar = ctk.CTkFrame(self, width=320, corner_radius=0)
        sidebar.grid(row=0, column=0, sticky="nsew")
        sidebar.grid_rowconfigure(2, weight=1)

        ctk.CTkLabel(sidebar, text="CodeContext AI", font=ctk.CTkFont(size=20, weight="bold")).grid(row=0, column=0,
                                                                                                    padx=20, pady=20)

        # Tabs
        self.tab_view = ctk.CTkTabview(sidebar)
        self.tab_view.grid(row=1, column=0, padx=10, pady=10, sticky="nsew")

        self.tab_run = self.tab_view.add("Run")
        self.tab_prompt = self.tab_view.add("Prompt")
        self.tab_settings = self.tab_view.add("Settings")

        # --- Tab: Run Elements ---
        ctk.CTkLabel(self.tab_run, text="Пресет:").pack(anchor="w", pady=(5, 0))
        self.cmb_preset = ctk.CTkComboBox(self.tab_run, values=list(PRESETS.keys()), command=self._apply_preset)
        self.cmb_preset.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(self.tab_run, text="Расширения:").pack(anchor="w")
        self.entry_ext = ctk.CTkEntry(self.tab_run)
        self.entry_ext.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(self.tab_run, text="Игнор папок:").pack(anchor="w")
        self.entry_ign = ctk.CTkEntry(self.tab_run)
        self.entry_ign.pack(fill="x", pady=(0, 10))

        self.chk_git = ctk.CTkCheckBox(self.tab_run, text="Только Git Changes")
        self.chk_git.pack(anchor="w", pady=5)

        self.chk_tree = ctk.CTkCheckBox(self.tab_run, text="Дерево файлов")
        self.chk_tree.pack(anchor="w", pady=5)

        ctk.CTkButton(self.tab_run, text="+ Папка", command=self._add_folder).pack(fill="x", pady=(20, 5))
        ctk.CTkButton(self.tab_run, text="Очистить", command=self._clear_folders, fg_color="transparent",
                      border_width=1).pack(fill="x", pady=5)

        # --- Tab: Prompt Elements ---
        self.txt_system_prompt = ctk.CTkTextbox(self.tab_prompt)
        self.txt_system_prompt.pack(fill="both", expand=True, pady=5)

        # --- Tab: Settings Elements ---
        ctk.CTkLabel(self.tab_settings, text="Интеграция с Windows", font=ctk.CTkFont(size=14, weight="bold")).pack(
            pady=(10, 5))
        ctk.CTkLabel(self.tab_settings, text="Добавляет пункт 'Scan with CodeContext AI'\nв контекстное меню папок.",
                     text_color="gray", font=("Arial", 11)).pack(pady=(0, 10))

        btn_install = ctk.CTkButton(self.tab_settings, text="Установить в контекстное меню", fg_color="green",
                                    command=self._install_context_menu)
        btn_install.pack(fill="x", pady=5)

        btn_remove = ctk.CTkButton(self.tab_settings, text="Удалить из меню", fg_color="red",
                                   command=self._remove_context_menu)
        btn_remove.pack(fill="x", pady=5)

        ctk.CTkLabel(self.tab_settings, text="Управление", font=ctk.CTkFont(size=14, weight="bold")).pack(pady=(20, 5))
        btn_reset = ctk.CTkButton(self.tab_settings, text="Сбросить настройки", fg_color="gray",
                                  command=self._reset_settings)
        btn_reset.pack(fill="x", pady=5)

        ctk.CTkLabel(self.tab_settings, text="v3.6 Clean Arch", text_color="gray").pack(side="bottom", pady=10)

        # === Main Area ===
        main_frame = ctk.CTkFrame(self, fg_color="transparent")
        main_frame.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)
        main_frame.grid_rowconfigure(3, weight=1)
        main_frame.grid_columnconfigure(0, weight=1)

        self.scroll_folders = ctk.CTkScrollableFrame(main_frame, height=120, label_text="Источники")
        self.scroll_folders.grid(row=0, column=0, sticky="ew", pady=(0, 10))

        opts = ctk.CTkFrame(main_frame, fg_color="transparent")
        opts.grid(row=1, column=0, sticky="ew", pady=(0, 10))

        self.chk_minify = ctk.CTkCheckBox(opts, text="Minify")
        self.chk_minify.pack(side="left", padx=10)
        self.chk_comments = ctk.CTkCheckBox(opts, text="No Comments")
        self.chk_comments.pack(side="left", padx=10)
        self.chk_secrets = ctk.CTkCheckBox(opts, text="No Secrets")
        self.chk_secrets.pack(side="left", padx=10)

        self.seg_format = ctk.CTkSegmentedButton(opts, values=["markdown", "xml", "plain"])
        self.seg_format.pack(side="right")
        self.seg_format.set("markdown")

        btns = ctk.CTkFrame(main_frame)
        btns.grid(row=2, column=0, sticky="ew", pady=(0, 10))

        ctk.CTkButton(btns, text="В Буфер", command=lambda: self._run_process("clipboard")).pack(side="left",
                                                                                                 expand=True, fill="x",
                                                                                                 padx=5, pady=5)
        ctk.CTkButton(btns, text="В Файл", command=lambda: self._run_process("file")).pack(side="left", expand=True,
                                                                                           fill="x", padx=5, pady=5)
        ctk.CTkButton(btns, text="В PDF", command=lambda: self._run_process("pdf")).pack(side="left", expand=True,
                                                                                         fill="x", padx=5, pady=5)

        self.txt_log = ctk.CTkTextbox(main_frame, font=("Consolas", 12))
        self.txt_log.grid(row=3, column=0, sticky="nsew")

        status_frame = ctk.CTkFrame(main_frame, height=30)
        status_frame.grid(row=4, column=0, sticky="ew", pady=(10, 0))

        self.progress_bar = ctk.CTkProgressBar(status_frame)
        self.progress_bar.pack(fill="x", side="top")
        self.progress_bar.set(0)

        self.lbl_status = ctk.CTkLabel(status_frame, text="Idle", text_color="gray")
        self.lbl_status.pack(side="left", padx=5)
        self.lbl_tokens = ctk.CTkLabel(status_frame, text="Tokens: 0", text_color="gray")
        self.lbl_tokens.pack(side="right", padx=5)

    def _load_initial_settings(self):
        data = self.settings_repo.load()
        if not data:
            data = {
                'extensions': PRESETS['Default']['ext'],
                'ignored_paths': PRESETS['Default']['ign'],
                'system_prompt': DEFAULT_SYSTEM_PROMPT
            }
        self.dispatcher.dispatch(SETTINGS_LOADED, data)

    def _on_store_changed(self, state):
        if self.entry_ext.get() != state.settings.extensions:
            self.entry_ext.delete(0, "end")
            self.entry_ext.insert(0, state.settings.extensions)

        if self.entry_ign.get() != state.settings.ignored_paths:
            self.entry_ign.delete(0, "end")
            self.entry_ign.insert(0, state.settings.ignored_paths)

        current_prompt = self.txt_system_prompt.get("1.0", "end-1c")
        if current_prompt != state.settings.system_prompt and not self.txt_system_prompt.focus_get():
            self.txt_system_prompt.delete("1.0", "end")
            self.txt_system_prompt.insert("1.0", state.settings.system_prompt)

        self._set_check(self.chk_minify, state.settings.minify)
        self._set_check(self.chk_comments, state.settings.remove_comments)
        self._set_check(self.chk_secrets, state.settings.remove_secrets)
        self._set_check(self.chk_tree, state.settings.include_tree)
        self._set_check(self.chk_git, state.settings.use_git)

        if len(self.scroll_folders.winfo_children()) != len(state.selected_folders):
            for w in self.scroll_folders.winfo_children():
                w.destroy()
            for folder in state.selected_folders:
                row = ctk.CTkFrame(self.scroll_folders)
                row.pack(fill="x", pady=2)
                ctk.CTkLabel(row, text=f"📂 {folder}", anchor="w").pack(side="left", padx=5)

        self.txt_log.configure(state="normal")
        self.txt_log.delete("1.0", "end")
        for log in state.logs:
            self.txt_log.insert("end", f"{log}\n")
        self.txt_log.see("end")
        self.txt_log.configure(state="disabled")

        self.lbl_status.configure(text=state.status_message)
        self.progress_bar.set(state.progress)
        self.lbl_tokens.configure(text=f"Tokens: {state.total_tokens}")

        state_state = "disabled" if state.is_loading else "normal"
        self.tab_view.configure(state=state_state)

    def _set_check(self, chk, val):
        if val:
            chk.select()
        else:
            chk.deselect()

    def _apply_preset(self, choice):
        preset = PRESETS.get(choice)
        if preset:
            self.dispatcher.dispatch(SETTINGS_UPDATE, {
                'extensions': preset['ext'],
                'ignored_paths': preset['ign']
            })

    def _add_folder(self):
        path = filedialog.askdirectory()
        if path:
            self.dispatcher.dispatch(FOLDER_ADD, path)

    def _clear_folders(self):
        self.dispatcher.dispatch(FOLDER_CLEAR)

    def _install_context_menu(self):
        try:
            success, msg = self.integration_service.install_context_menu()
            if success:
                messagebox.showinfo("Успех", msg)
                self.dispatcher.dispatch(UI_ADD_LOG, f"System: {msg}")
            else:
                messagebox.showerror("Ошибка", msg)
        except SystemExit:
            pass

    def _remove_context_menu(self):
        try:
            success, msg = self.integration_service.remove_context_menu()
            if success:
                messagebox.showinfo("Успех", msg)
                self.dispatcher.dispatch(UI_ADD_LOG, f"System: {msg}")
            else:
                messagebox.showerror("Ошибка", msg)
        except SystemExit:
            pass

    def _reset_settings(self):
        if messagebox.askyesno("Сброс", "Сбросить настройки к значениям по умолчанию?"):
            default_data = {
                'extensions': PRESETS['Default']['ext'],
                'ignored_paths': PRESETS['Default']['ign'],
                'system_prompt': DEFAULT_SYSTEM_PROMPT,
                'minify': True,
                'remove_comments': True,
                'remove_secrets': True,
                'include_tree': True
            }
            self.dispatcher.dispatch(SETTINGS_UPDATE, default_data)
            self.settings_repo.save(default_data)
            self.dispatcher.dispatch(UI_ADD_LOG, "Настройки сброшены")

    def _run_process(self, target):
        self.dispatcher.dispatch(SETTINGS_UPDATE, {
            'extensions': self.entry_ext.get(),
            'ignored_paths': self.entry_ign.get(),
            'minify': bool(self.chk_minify.get()),
            'remove_comments': bool(self.chk_comments.get()),
            'remove_secrets': bool(self.chk_secrets.get()),
            'include_tree': bool(self.chk_tree.get()),
            'use_git': bool(self.chk_git.get()),
            'system_prompt': self.txt_system_prompt.get("1.0", "end-1c"),
            'output_format': self.seg_format.get()
        })

        if not self.store.state.selected_folders:
            messagebox.showwarning("Внимание", "Выберите папки для сканирования")
            return

        threading.Thread(target=self._worker, args=(target,), daemon=True).start()

    def _worker(self, target):
        """
        ОРКЕСТРАЦИЯ ПРОЦЕССА (Controller / Thunk)
        Здесь происходит вызов сервисов в нужном порядке.
        """
        self.dispatcher.dispatch(UI_SET_LOADING, True)
        self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сканирование...", 'progress': 0.1})
        self.dispatcher.dispatch(UI_ADD_LOG, "Начало работы...")

        state = self.store.state

        try:
            # 1. СКАНИРОВАНИЕ (Получаем список путей)
            files_paths = self.file_service.scan_folders(
                state.selected_folders,
                state.settings.extensions,
                state.settings.ignored_paths,
                state.settings.use_git
            )

            if not files_paths:
                self.dispatcher.dispatch(SCAN_FAILURE, "Файлы не найдены")
                self.dispatcher.dispatch(UI_SET_LOADING, False)
                return

            self.dispatcher.dispatch(SCAN_SUCCESS, files_paths)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Чтение файлов...", 'progress': 0.2})

            # 2. ЧТЕНИЕ (ProcessingService только читает, не чистит)
            raw_files = self.process_service.read_files(files_paths)

            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Обработка...", 'progress': 0.4})

            # 3. ОБРАБОТКА (Оркестрация Cleaning + Tokenizing в цикле)
            processed_results = []

            for i, raw_file in enumerate(raw_files):
                # Обновляем прогресс каждые 10 файлов
                if i % 10 == 0:
                    prog = 0.4 + (0.4 * (i / len(raw_files)))
                    self.dispatcher.dispatch(UI_UPDATE_STATUS,
                                             {'message': f"Обработка {i}/{len(raw_files)}...", 'progress': prog})

                content = raw_file['content']
                ext = raw_file['ext']

                # A. Cleaning
                cleaned_content = self.cleaner_service.clean(content, ext, state.settings)

                # B. Tokenizing
                tokens = self.token_service.count_tokens(cleaned_content)

                # C. Create DTO
                processed_results.append(ProcessedFile(
                    path=raw_file['path'],
                    content=cleaned_content,
                    tokens=tokens
                ))

            self.dispatcher.dispatch(PROCESSING_SUCCESS, processed_results)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Форматирование...", 'progress': 0.9})

            # 4. ФОРМАТИРОВАНИЕ
            text_result = self.format_service.format_output(
                processed_results,
                state.settings.output_format,
                state.settings.include_tree,
                state.settings.system_prompt
            )

            total_tokens = sum(f.tokens for f in processed_results)
            self.dispatcher.dispatch(FORMATTING_SUCCESS, {'text': text_result, 'tokens': total_tokens})

            # 5. ВЫВОД (Output)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Сохранение...", 'progress': 0.95})

            if target == 'clipboard':
                self.output_service.copy_to_clipboard(text_result)
                self.dispatcher.dispatch(UI_ADD_LOG, "Скопировано в буфер обмена")
            elif target == 'file':
                ext = f".{state.settings.output_format}" if state.settings.output_format != 'plain' else ".txt"
                path = filedialog.asksaveasfilename(defaultextension=ext)
                if path:
                    self.output_service.save_to_file(text_result, path)
                    self.dispatcher.dispatch(UI_ADD_LOG, f"Сохранено в {path}")
            elif target == 'pdf':
                path = filedialog.asksaveasfilename(defaultextension=".pdf")
                if path:
                    self.output_service.save_to_pdf(text_result, path)
                    self.dispatcher.dispatch(UI_ADD_LOG, f"PDF создан: {path}")

            self.settings_repo.save(state.settings.__dict__)

        except Exception as e:
            self.dispatcher.dispatch(UI_ADD_LOG, f"CRITICAL ERROR: {e}")
            import traceback
            traceback.print_exc()
        finally:
            self.dispatcher.dispatch(UI_SET_LOADING, False)
            self.dispatcher.dispatch(UI_UPDATE_STATUS, {'message': "Готово", 'progress': 1.0})

    def on_closing(self):
        if self.unsubscribe:
            self.unsubscribe()
        self.destroy()