"""
Main application window UI for the Code Aggregator application with modern CustomTkinter interface.
"""
import os
import sys
import threading
import customtkinter as ctk
from tkinter import filedialog, messagebox
from pathlib import Path
from typing import List, Set, Callable, Optional
import pyperclip
from src.logic.file_scanner import FileScanner
from src.services.pdf_generator import PdfService
from src.services.token_service import TokenService
from src.config import DEFAULT_EXTENSIONS, DEFAULT_IGNORED


# Set appearance mode and theme for CustomTkinter
ctk.set_appearance_mode("Dark")  # Options: "System", "Dark", "Light"
ctk.set_default_color_theme("blue")  # Options: "blue", "green", "dark-blue"


class App(ctk.CTk):
    """
    Main application window class with modern CustomTkinter UI.
    """
    def __init__(self):
        super().__init__()

        # Basic window settings
        self.title("CodeContext AI v2.0")
        self.geometry("1100x800")
        self.minsize(800, 600)

        # Application state
        self.selected_folders: List[str] = []
        self.scanner: Optional[FileScanner] = None
        self.token_service = TokenService()
        self.pdf_service = PdfService()
        self.processing = False

        self._setup_layout()

    def _setup_layout(self):
        """Create grid layout and widgets."""
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # --- Sidebar (Left panel) ---
        self.sidebar = ctk.CTkFrame(self, width=250, corner_radius=0)
        self.sidebar.grid(row=0, column=0, sticky="nsew")
        self.sidebar.grid_rowconfigure(7, weight=1) # spacer

        logo = ctk.CTkLabel(self.sidebar, text="CodeContext AI", font=ctk.CTkFont(size=20, weight="bold"))
        logo.grid(row=0, column=0, padx=20, pady=(20, 10))

        # File extensions settings
        ctk.CTkLabel(self.sidebar, text="Расширения файлов:", anchor="w").grid(row=1, column=0, padx=20, pady=(10, 0), sticky="w")
        self.entry_ext = ctk.CTkEntry(self.sidebar, placeholder_text=".py .js ...")
        self.entry_ext.grid(row=2, column=0, padx=20, pady=(0, 10), sticky="ew")
        self.entry_ext.insert(0, DEFAULT_EXTENSIONS)

        # Ignore directories settings
        ctk.CTkLabel(self.sidebar, text="Игнорировать папки:", anchor="w").grid(row=3, column=0, padx=20, pady=(10, 0), sticky="w")
        self.entry_ignore = ctk.CTkEntry(self.sidebar, placeholder_text="node_modules...")
        self.entry_ignore.grid(row=4, column=0, padx=20, pady=(0, 20), sticky="ew")
        self.entry_ignore.insert(0, DEFAULT_IGNORED)

        # Folder management buttons
        self.btn_add = ctk.CTkButton(self.sidebar, text="+ Добавить папку", command=self.add_folder)
        self.btn_add.grid(row=5, column=0, padx=20, pady=5)

        self.btn_clear = ctk.CTkButton(self.sidebar, text="Очистить список", fg_color="transparent", border_width=2, command=self.clear_folders)
        self.btn_clear.grid(row=6, column=0, padx=20, pady=5)

        # Footer sidebar
        lbl_ver = ctk.CTkLabel(self.sidebar, text="v2.0 Production Ready", text_color="gray")
        lbl_ver.grid(row=8, column=0, padx=20, pady=20)

        # --- Main Area (Right panel) ---
        self.main_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.main_frame.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)
        self.main_frame.grid_rowconfigure(1, weight=1) # Log area expands
        self.main_frame.grid_columnconfigure(0, weight=1)

        # 1. Folder list (Scrollable)
        lbl_folders = ctk.CTkLabel(self.main_frame, text="Выбранные директории источника", font=ctk.CTkFont(size=14, weight="bold"))
        lbl_folders.grid(row=0, column=0, sticky="w", pady=(0, 5))

        self.folder_scroll = ctk.CTkScrollableFrame(self.main_frame, height=150, label_text="Список путей")
        self.folder_scroll.grid(row=1, column=0, sticky="nsew", pady=(0, 20))

        # 2. Action panel
        self.action_panel = ctk.CTkFrame(self.main_frame)
        self.action_panel.grid(row=2, column=0, sticky="ew", pady=(0, 20))

        # Action buttons
        self.btn_copy = ctk.CTkButton(self.action_panel, text="📋 В Буфер", command=lambda: self.start_process('clipboard'), height=40, font=ctk.CTkFont(weight="bold"))
        self.btn_copy.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        self.btn_txt = ctk.CTkButton(self.action_panel, text="💾 Сохранить .txt", command=lambda: self.start_process('txt'), height=40)
        self.btn_txt.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        self.btn_pdf = ctk.CTkButton(self.action_panel, text="📄 Создать PDF", command=lambda: self.start_process('pdf'), height=40)
        self.btn_pdf.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        # 3. Logs and Status
        self.log_box = ctk.CTkTextbox(self.main_frame, font=("Consolas", 12), activate_scrollbars=True)
        self.log_box.grid(row=3, column=0, sticky="nsew")
        self.log_box.insert("0.0", "Ожидание действий...\n")
        self.log_box.configure(state="disabled")

        self.status_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        self.status_frame.grid(row=4, column=0, sticky="ew", pady=(10, 0))

        self.progress_bar = ctk.CTkProgressBar(self.status_frame)
        self.progress_bar.pack(fill="x", pady=(0, 5))
        self.progress_bar.set(0)

        self.lbl_status = ctk.CTkLabel(self.status_frame, text="Готово", anchor="w")
        self.lbl_status.pack(side="left")

        self.lbl_tokens = ctk.CTkLabel(self.status_frame, text="Токены: 0", anchor="e")
        self.lbl_tokens.pack(side="right")

    # --- UI Logic ---

    def add_folder(self):
        """Add a folder to the list of selected folders."""
        folder = filedialog.askdirectory()
        if folder:
            if folder in self.selected_folders:
                return
            self.selected_folders.append(folder)

            # Visual addition
            row_frame = ctk.CTkFrame(self.folder_scroll)
            row_frame.pack(fill="x", pady=2)

            lbl = ctk.CTkLabel(row_frame, text=f"📂 {folder}", anchor="w")
            lbl.pack(side="left", padx=5)

            self.log(f"[INFO] Добавлена папка: {folder}")

    def clear_folders(self):
        """Clear the list of selected folders."""
        self.selected_folders.clear()
        for widget in self.folder_scroll.winfo_children():
            widget.destroy()
        self.log("[INFO] Список папок очищен")

    def log(self, message: str):
        """Add a message to the log area."""
        self.log_box.configure(state="normal")
        self.log_box.insert("end", f"{message}\n")
        self.log_box.see("end")
        self.log_box.configure(state="disabled")

    def toggle_inputs(self, enable: bool):
        """Enable or disable all input controls."""
        state = "normal" if enable else "disabled"
        self.btn_copy.configure(state=state)
        self.btn_txt.configure(state=state)
        self.btn_pdf.configure(state=state)
        self.btn_add.configure(state=state)
        self.btn_clear.configure(state=state)

    def update_progress(self, message: str, val: float):
        """Update progress bar and status message."""
        self.lbl_status.configure(text=message)
        self.progress_bar.set(val)

    # --- Processing Flow ---

    def start_process(self, mode: str):
        """Start the processing in a separate thread."""
        if not self.selected_folders:
            messagebox.showwarning("Ошибка", "Выберите хотя бы одну папку!")
            return

        if self.processing:
            return
        self.processing = True
        self.toggle_inputs(False)
        self.log_box.configure(state="normal")
        self.log_box.delete("1.0", "end")
        self.log_box.configure(state="disabled")

        # Read settings from UI
        ext_str = self.entry_ext.get()
        ign_str = self.entry_ignore.get()

        ext_list = ext_str.replace(",", " ").split()
        ign_set = {x.strip() for x in ign_str.split(",")}

        self.scanner = FileScanner(ext_list, ign_set)

        # Start in separate thread
        thread = threading.Thread(target=self._run_heavy_task, args=(mode,), daemon=True)
        thread.start()

    def _run_heavy_task(self, mode: str):
        """Run the heavy processing task in a separate thread."""
        try:
            # 1. Scanning and aggregation
            def cb(msg, prog):
                # Update UI from another thread - CustomTkinter should handle this
                self.update_progress(msg, prog)

            self.log("--- Начало сканирования ---")
            full_text = self.scanner.scan_and_process(self.selected_folders, cb)

            if not full_text:
                self.log("[WARN] Файлы не найдены.")
                self.end_process()
                return

            # 2. Token counting
            self.update_progress("Подсчет токенов...", 0.9)
            token_count = self.token_service.count(full_text)
            self.lbl_tokens.configure(text=f"Токены (GPT-4): ~{token_count}")
            self.log(f"[STATS] Найдено символов: {len(full_text)}")
            self.log(f"[STATS] Примерно токенов: {token_count}")

            # 3. Action based on mode
            if mode == 'clipboard':
                pyperclip.copy(full_text)
                self.log("[SUCCESS] Скопировано в буфер обмена!")
                messagebox.showinfo("Готово", "Код скопирован в буфер обмена.")

            elif mode == 'txt':
                file_path = filedialog.asksaveasfilename(defaultextension=".txt", filetypes=[("Text", "*.txt")])
                if file_path:
                    Path(file_path).write_text(full_text, encoding='utf-8')
                    self.log(f"[SUCCESS] Сохранено в {file_path}")

            elif mode == 'pdf':
                file_path = filedialog.asksaveasfilename(defaultextension=".pdf", filetypes=[("PDF", "*.pdf")])
                if file_path:
                    self.update_progress("Генерация PDF...", 0.95)
                    self.pdf_service.create_pdf(full_text, file_path, self.log)

        except Exception as e:
            self.log(f"[ERROR] {e}")
            print(e)
        finally:
            self.end_process()

    def end_process(self):
        """End the processing and reset UI state."""
        self.processing = False
        self.toggle_inputs(True)
        self.update_progress("Готово", 1.0)
        self.log("--- Операция завершена ---")