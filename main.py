"""CodeContext AI v3.0 Pro - Main Application."""
import os
import sys
import threading
import re
import html
import platform
import customtkinter as ctk
from tkinter import filedialog, messagebox
from pathlib import Path
from typing import List, Set, Callable, Dict, Optional
import pyperclip

from src.services.token_service import TokenService
from src.services.code_cleaner import CodeCleaner
from src.services.formatter_service import FormatterService
from src.services.pdf_service import PdfService
from src.services.file_scanner import FileScanner
from src.config import DEFAULT_EXTENSIONS, DEFAULT_IGNORED, MAX_FILE_SIZE_MB


# =================================================================================
# CONSTANTS & CONFIG
# =================================================================================

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")


# =================================================================================
# CONTROLLER & VIEW (UI Layer)
# =================================================================================

class ModernApp(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.title("CodeContext AI v3.0 Pro")
        self.geometry("1100x850")
        
        # Services
        self.token_service = TokenService()
        self.cleaner_service = CodeCleaner()
        
        # State
        self.selected_folders: List[str] = []
        self.processing = False
        
        self._init_ui()

    def _init_ui(self):
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # --- LEFT SIDEBAR (Config) ---
        sidebar = ctk.CTkFrame(self, width=280, corner_radius=0)
        sidebar.grid(row=0, column=0, sticky="nsew")
        sidebar.grid_rowconfigure(10, weight=1)

        ctk.CTkLabel(sidebar, text="Настройки источника", font=ctk.CTkFont(size=18, weight="bold")).grid(row=0, column=0, padx=20, pady=(20, 10))
        
        self.entry_ext = ctk.CTkEntry(sidebar, placeholder_text="Расширения")
        self.entry_ext.insert(0, DEFAULT_EXTENSIONS)
        self.entry_ext.grid(row=1, column=0, padx=20, pady=(0, 10), sticky="ew")

        self.entry_ign = ctk.CTkEntry(sidebar, placeholder_text="Игнор папок")
        self.entry_ign.insert(0, DEFAULT_IGNORED)
        self.entry_ign.grid(row=2, column=0, padx=20, pady=(0, 20), sticky="ew")

        ctk.CTkButton(sidebar, text="Добавить папку", command=self.add_folder).grid(row=3, column=0, padx=20, pady=5)
        ctk.CTkButton(sidebar, text="Очистить все", fg_color="transparent", border_width=1, command=self.clear_folders).grid(row=4, column=0, padx=20, pady=5)

        # --- OPTIONS SECTION (New) ---
        ctk.CTkLabel(sidebar, text="Опции обработки", font=ctk.CTkFont(size=16, weight="bold")).grid(row=5, column=0, padx=20, pady=(30, 10))
        
        self.chk_minify = ctk.CTkCheckBox(sidebar, text="Minify (убрать пробелы)")
        self.chk_minify.grid(row=6, column=0, padx=20, pady=5, sticky="w")
        
        self.chk_comments = ctk.CTkCheckBox(sidebar, text="Убрать комментарии (Beta)")
        self.chk_comments.grid(row=7, column=0, padx=20, pady=5, sticky="w")

        ctk.CTkLabel(sidebar, text="Формат вывода:", anchor="w").grid(row=8, column=0, padx=20, pady=(20, 5), sticky="w")
        self.format_var = ctk.StringVar(value="Markdown")
        self.seg_format = ctk.CTkSegmentedButton(sidebar, values=["Plain", "Markdown", "XML"], variable=self.format_var)
        self.seg_format.grid(row=9, column=0, padx=20, pady=0)

        # --- MAIN AREA ---
        main = ctk.CTkFrame(self, fg_color="transparent")
        main.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)
        main.grid_rowconfigure(2, weight=1)
        main.grid_columnconfigure(0, weight=1)

        # 1. Folder List
        self.scroll_folders = ctk.CTkScrollableFrame(main, height=120, label_text="Выбранные пути")
        self.scroll_folders.grid(row=0, column=0, sticky="ew", pady=(0, 15))

        # 2. Action Buttons
        btn_frame = ctk.CTkFrame(main)
        btn_frame.grid(row=1, column=0, sticky="ew", pady=(0, 15))
        
        self.btn_clip = ctk.CTkButton(btn_frame, text="📋 В Буфер", font=ctk.CTkFont(weight="bold"), height=40, command=lambda: self.run_task('clipboard'))
        self.btn_clip.pack(side="left", padx=10, pady=10, expand=True, fill="x")
        
        self.btn_file = ctk.CTkButton(btn_frame, text="💾 В Файл", height=40, command=lambda: self.run_task('file'))
        self.btn_file.pack(side="left", padx=10, pady=10, expand=True, fill="x")
        
        self.btn_pdf = ctk.CTkButton(btn_frame, text="📄 В PDF", height=40, command=lambda: self.run_task('pdf'))
        self.btn_pdf.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        # 3. Logs & Preview
        self.log_box = ctk.CTkTextbox(main, font=("Consolas", 12))
        self.log_box.grid(row=2, column=0, sticky="nsew")
        self.log_box.insert("0.0", "Готов к работе. Выберите папки и формат вывода.\n")

        # 4. Status Bar
        self.status_frame = ctk.CTkFrame(main, height=30)
        self.status_frame.grid(row=3, column=0, sticky="ew", pady=(10, 0))
        
        self.progress = ctk.CTkProgressBar(self.status_frame)
        self.progress.pack(fill="x", side="top")
        self.progress.set(0)
        
        self.lbl_stat = ctk.CTkLabel(self.status_frame, text="Idle", text_color="gray")
        self.lbl_stat.pack(side="left", padx=5)
        
        self.lbl_token = ctk.CTkLabel(self.status_frame, text="Tokens: 0", text_color="gray")
        self.lbl_token.pack(side="right", padx=5)

    # --- UI Logic ---

    def add_folder(self):
        path = filedialog.askdirectory()
        if path and path not in self.selected_folders:
            self.selected_folders.append(path)
            l = ctk.CTkLabel(self.scroll_folders, text=f"📂 {path}", anchor="w")
            l.pack(fill="x", padx=5)

    def clear_folders(self):
        self.selected_folders.clear()
        for w in self.scroll_folders.winfo_children(): 
            w.destroy()

    def log(self, msg):
        self.log_box.configure(state="normal")
        self.log_box.insert("end", f"{msg}\n")
        self.log_box.see("end")
        self.log_box.configure(state="disabled")

    def update_status(self, msg, progress):
        self.lbl_stat.configure(text=msg)
        self.progress.set(progress)

    def lock_ui(self, lock):
        state = "disabled" if lock else "normal"
        self.btn_clip.configure(state=state)
        self.btn_file.configure(state=state)
        self.btn_pdf.configure(state=state)

    # --- BUSINESS LOGIC ---

    def run_task(self, target):
        if not self.selected_folders:
            messagebox.showwarning("Ошибка", "Нет папок для обработки")
            return
        
        self.lock_ui(True)
        self.log_box.configure(state="normal")
        self.log_box.delete("1.0", "end")
        self.log_box.configure(state="disabled")
        
        # Сбор параметров
        params = {
            'exts': self.entry_ext.get().split(),
            'ign': {x.strip() for x in self.entry_ign.get().split(',')},
            'minify': bool(self.chk_minify.get()),
            'remove_comments': bool(self.chk_comments.get()),
            'format': self.format_var.get().lower(), # markdown, xml, plain
            'target': target
        }
        
        threading.Thread(target=self._worker, args=(params,), daemon=True).start()

    def _worker(self, params):
        try:
            # 1. Scanning
            file_entries = []
            files_to_process = []
            
            # Create scanner
            scanner = FileScanner(params['exts'], params['ign'])
            files_to_process = scanner.scan(self.selected_folders)
            
            total = len(files_to_process)
            if total == 0:
                self.log("❌ Файлы не найдены.")
                return

            # 2. Reading & Cleaning
            processed_count = 0
            for path in files_to_process:
                try:
                    if path.stat().st_size > MAX_FILE_SIZE_MB * 1024 * 1024:
                        self.log(f"⚠️ Пропуск (большой файл): {path.name}")
                        continue
                        
                    raw_content = path.read_text(encoding='utf-8', errors='replace')
                    
                    # Применяем очистку
                    clean_content = self.cleaner_service.process(
                        raw_content, 
                        path.suffix, 
                        params
                    )
                    
                    file_entries.append({
                        'path': str(path),
                        'content': clean_content
                    })
                except Exception as e:
                    print(f"Err {path}: {e}")
                
                processed_count += 1
                if processed_count % 5 == 0:
                    prog = 0.1 + (0.6 * (processed_count / total))
                    self.update_status(f"Обработка: {path.name}", prog)

            # 3. Formatting
            self.update_status("Форматирование...", 0.8)
            fmt = params['format']
            
            if fmt == 'xml':
                result_text = FormatterService.to_xml(file_entries)
            elif fmt == 'markdown':
                result_text = FormatterService.to_markdown(file_entries)
            else:
                result_text = FormatterService.to_plain(file_entries)

            # 4. Token Counting
            self.update_status("Считаем токены...", 0.9)
            tokens = self.token_service.count(result_text)
            self.lbl_token.configure(text=f"Tokens: ~{tokens}")

            # 5. Exporting
            target = params['target']
            
            if target == 'clipboard':
                pyperclip.copy(result_text)
                self.log(f"✅ Успешно! {len(file_entries)} файлов.\nСкопировано в буфер.")
            
            elif target == 'file':
                # В CustomTkinter/Tkinter вызов диалога из потока на Windows работает, на Mac нужен фикс
                # Для надежности используем after, но здесь упростим
                ext = f".{fmt}" if fmt != 'plain' else ".txt"
                path = filedialog.asksaveasfilename(defaultextension=ext)
                if path:
                    Path(path).write_text(result_text, encoding='utf-8')
                    self.log(f"✅ Сохранено в: {path}")

            elif target == 'pdf':
                path = filedialog.asksaveasfilename(defaultextension=".pdf")
                if path:
                    PdfService.create_pdf(result_text, path)
                    self.log(f"✅ PDF создан: {path}")

            self.update_status("Готово", 1.0)

        except Exception as e:
            self.log(f"🔥 Критическая ошибка: {e}")
        finally:
            self.lock_ui(False)

if __name__ == "__main__":
    app = ModernApp()
    app.mainloop()