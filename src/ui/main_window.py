"""
Main application window UI for the Code Aggregator application.
"""
import os
import sys
import threading
import tkinter as tk
from tkinter import filedialog, messagebox, ttk, scrolledtext
from pathlib import Path
from typing import List, Set
import pyperclip
from src.logic.file_scanner import FileScanner
from src.logic.content_aggregator import ContentAggregator
from src.services.pdf_generator import PdfGenerator
from src.config import DEFAULT_EXTENSIONS, IGNORED_DIRS


class App(tk.Tk):
    """
    Main application window class.
    """
    def __init__(self):
        super().__init__()
        self.title("Code Aggregator Pro")
        self.geometry("900x700")
        
        # State
        self.selected_folders: List[str] = []
        
        self._setup_ui()
        self._setup_styles()

    def _setup_styles(self):
        """
        Setup custom styles for the UI elements.
        """
        style = ttk.Style(self)
        style.theme_use('clam')
        style.configure("TButton", font=("Segoe UI", 10), padding=5)
        style.configure("TLabel", font=("Segoe UI", 10))

    def _setup_ui(self):
        """
        Setup the user interface components.
        """
        # 1. Top Panel (Configuration)
        top_frame = ttk.LabelFrame(self, text="Настройки поиска", padding=(10, 5))
        top_frame.pack(fill="x", padx=10, pady=5)

        ttk.Label(top_frame, text="Расширения файлов:").pack(anchor="w")
        self.ext_entry = ttk.Entry(top_frame)
        self.ext_entry.insert(0, " ".join(DEFAULT_EXTENSIONS))
        self.ext_entry.pack(fill="x", pady=(0, 10))

        ttk.Label(top_frame, text="Игнорировать папки (через запятую):").pack(anchor="w")
        self.ignore_entry = ttk.Entry(top_frame)
        self.ignore_entry.insert(0, ", ".join(IGNORED_DIRS))
        self.ignore_entry.pack(fill="x", pady=(0, 5))

        # 2. Middle Panel (Folders List)
        mid_frame = ttk.LabelFrame(self, text="Выбранные папки", padding=(10, 5))
        mid_frame.pack(fill="both", expand=True, padx=10, pady=5)

        btn_frame = ttk.Frame(mid_frame)
        btn_frame.pack(fill="x", pady=5)
        
        ttk.Button(btn_frame, text="+ Добавить папку", command=self.add_folder).pack(side="left", padx=5)
        ttk.Button(btn_frame, text="Очистить список", command=self.clear_folders).pack(side="left", padx=5)

        self.listbox = tk.Listbox(mid_frame, selectmode=tk.SINGLE, height=6)
        self.listbox.pack(fill="both", expand=True, padx=5, pady=5)

        # 3. Action Panel
        action_frame = ttk.Frame(self, padding=(10, 5))
        action_frame.pack(fill="x", padx=10)

        ttk.Button(action_frame, text="📋 Копировать в буфер", command=lambda: self.run_action('clipboard')).pack(side="left", padx=5, expand=True, fill="x")
        ttk.Button(action_frame, text="💾 Сохранить в .txt", command=lambda: self.run_action('txt')).pack(side="left", padx=5, expand=True, fill="x")
        ttk.Button(action_frame, text="📄 Сохранить в .pdf", command=lambda: self.run_action('pdf')).pack(side="left", padx=5, expand=True, fill="x")

        # 4. Logs
        log_frame = ttk.LabelFrame(self, text="Логи выполнения", padding=(10, 5))
        log_frame.pack(fill="both", expand=True, padx=10, pady=10)

        self.log_area = scrolledtext.ScrolledText(log_frame, height=10, state='disabled', font=("Consolas", 9))
        self.log_area.pack(fill="both", expand=True)

    # --- UI Events ---

    def log(self, message: str):
        """
        Add a message to the log area.
        
        Args:
            message: Message to add to the log
        """
        self.log_area.config(state='normal')
        self.log_area.insert(tk.END, message + "\n")
        self.log_area.see(tk.END)
        self.log_area.config(state='disabled')

    def add_folder(self):
        """
        Add a folder to the list of selected folders.
        """
        folder = filedialog.askdirectory()
        if folder:
            if folder not in self.selected_folders:
                self.selected_folders.append(folder)
                self.listbox.insert(tk.END, folder)
                self.log(f"[INFO] Добавлена папка: {folder}")

    def clear_folders(self):
        """
        Clear the list of selected folders.
        """
        self.selected_folders = []
        self.listbox.delete(0, tk.END)
        self.log("[INFO] Список папок очищен")

    def run_action(self, action_type: str):
        """
        Run the selected action (clipboard, txt, pdf).
        
        Args:
            action_type: Type of action to perform ('clipboard', 'txt', 'pdf')
        """
        if not self.selected_folders:
            messagebox.showwarning("Внимание", "Выберите хотя бы одну папку!")
            return

        # Parse config from UI
        raw_exts = self.ext_entry.get().split()
        raw_ignores = {x.strip() for x in self.ignore_entry.get().split(",")}

        # Run in separate thread to prevent GUI from freezing
        thread = threading.Thread(
            target=self._process_data, 
            args=(action_type, raw_exts, raw_ignores),
            daemon=True
        )
        thread.start()

    # --- Business Logic Binding ---

    def _process_data(self, action_type: str, extensions: List[str], ignores: Set[str]):
        """
        Process the data based on the selected action type.
        
        Args:
            action_type: Type of action to perform ('clipboard', 'txt', 'pdf')
            extensions: List of file extensions to include
            ignores: Set of directory names to ignore
        """
        self.log(f"--- Старт операции: {action_type.upper()} ---")
        
        scanner = FileScanner(extensions, ignores)
        aggregator = ContentAggregator()

        # 1. Scan and read
        files_gen = scanner.scan(self.selected_folders, self.log)
        full_text = aggregator.aggregate(files_gen, self.log)

        if not full_text:
            self.log("[WARN] Файлы не найдены или результат пуст.")
            return

        # 2. Perform action
        try:
            if action_type == 'clipboard':
                pyperclip.copy(full_text)
                self.log("[SUCCESS] Результат скопирован в буфер обмена!")
                messagebox.showinfo("Готово", "Текст скопирован в буфер обмена.")
            
            elif action_type == 'txt':
                file_path = filedialog.asksaveasfilename(
                    defaultextension=".txt", 
                    filetypes=[("Text files", "*.txt")],
                    title="Сохранить результат"
                )
                if file_path:
                    Path(file_path).write_text(full_text, encoding='utf-8')
                    self.log(f"[SUCCESS] Сохранено в: {file_path}")
            
            elif action_type == 'pdf':
                file_path = filedialog.asksaveasfilename(
                    defaultextension=".pdf", 
                    filetypes=[("PDF files", "*.pdf")],
                    title="Сохранить PDF"
                )
                if file_path:
                    self.log("[INFO] Генерация PDF может занять время...")
                    PdfGenerator.create_pdf(full_text, file_path, self.log)
                    
        except Exception as e:
            self.log(f"[ERROR] Critical error: {e}")
            messagebox.showerror("Ошибка", str(e))
            
        self.log("--- Операция завершена ---\n")