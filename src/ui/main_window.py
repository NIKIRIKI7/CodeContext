"""
CodeContext AI v3.6 - Main Window UI
"""
import threading
import customtkinter as ctk
from tkinter import filedialog, messagebox
from pathlib import Path
from typing import List
import pyperclip

from src.services.token_service import TokenService
from src.services.code_cleaner import CodeCleaner
from src.services.formatter_service import FormatterService
from src.services.pdf_service import PdfService
from src.services.file_scanner import FileScanner
from src.services.settings_manager import SettingsManager
from src.config import PRESETS, DEFAULT_SYSTEM_PROMPT, MAX_FILE_SIZE_MB

ctk.set_appearance_mode("Dark")
ctk.set_default_color_theme("blue")

class ModernApp(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.title("CodeContext AI v3.6 Pro")
        self.geometry("1150x850")

        self.token_service = TokenService()
        self.cleaner_service = CodeCleaner()
        self.selected_folders: List[str] = []

        self._init_ui()

    def _init_ui(self):
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # --- Sidebar ---
        sidebar = ctk.CTkFrame(self, width=320, corner_radius=0)
        sidebar.grid(row=0, column=0, sticky="nsew")
        sidebar.grid_rowconfigure(2, weight=1)

        ctk.CTkLabel(sidebar, text="CodeContext AI", font=ctk.CTkFont(size=20, weight="bold")).grid(row=0, column=0, padx=20, pady=(20, 10))

        # Tabs
        self.tab_view = ctk.CTkTabview(sidebar, width=280)
        self.tab_view.grid(row=1, column=0, padx=10, pady=10, sticky="nsew")
        self.tab_view.add("Run")
        self.tab_view.add("Prompt")
        self.tab_view.add("CLI/Config")

        # === TAB 1: RUN (Текущий запуск) ===
        tab_run = self.tab_view.tab("Run")

        ctk.CTkLabel(tab_run, text="Пресет:", anchor="w").pack(fill="x", pady=(5, 0))
        self.cmb_preset = ctk.CTkComboBox(tab_run, values=list(PRESETS.keys()), command=self.apply_preset)
        self.cmb_preset.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(tab_run, text="Расширения:", anchor="w").pack(fill="x")
        self.entry_ext = ctk.CTkEntry(tab_run)
        self.entry_ext.insert(0, PRESETS["Default"]["ext"])
        self.entry_ext.pack(fill="x", pady=(0, 10))

        ctk.CTkLabel(tab_run, text="Игнор папок:", anchor="w").pack(fill="x")
        self.entry_ign = ctk.CTkEntry(tab_run)
        self.entry_ign.insert(0, PRESETS["Default"]["ign"])
        self.entry_ign.pack(fill="x", pady=(0, 10))

        self.chk_git = ctk.CTkCheckBox(tab_run, text="Только Git Changes")
        self.chk_git.pack(anchor="w", pady=5)

        self.chk_tree = ctk.CTkCheckBox(tab_run, text="Дерево файлов")
        self.chk_tree.select()
        self.chk_tree.pack(anchor="w", pady=5)

        ctk.CTkButton(tab_run, text="+ Папка", command=self.add_folder).pack(fill="x", pady=(20, 5))
        ctk.CTkButton(tab_run, text="Очистить", fg_color="transparent", border_width=1, command=self.clear_folders).pack(fill="x", pady=5)

        # === TAB 2: PROMPT (Для текущего запуска) ===
        tab_prompt = self.tab_view.tab("Prompt")
        self.txt_prompt = ctk.CTkTextbox(tab_prompt, height=300)
        self.txt_prompt.pack(fill="both", expand=True, pady=5)
        self.txt_prompt.insert("0.0", DEFAULT_SYSTEM_PROMPT)

        # === TAB 3: CLI CONFIG (Глобальные настройки для ПКМ) ===
        tab_cli = self.tab_view.tab("CLI/Config")

        ctk.CTkLabel(tab_cli, text="Настройки для запуска\nчерез правую кнопку мыши:", text_color="gray").pack(pady=(0, 10))

        self.chk_cli_minify = ctk.CTkCheckBox(tab_cli, text="Minify (сжать)")
        self.chk_cli_minify.pack(anchor="w", pady=2)

        self.chk_cli_comments = ctk.CTkCheckBox(tab_cli, text="Без комментариев")
        self.chk_cli_comments.pack(anchor="w", pady=2)

        self.chk_cli_secrets = ctk.CTkCheckBox(tab_cli, text="Скрывать секреты")
        self.chk_cli_secrets.pack(anchor="w", pady=2)

        self.chk_cli_tree = ctk.CTkCheckBox(tab_cli, text="Включать дерево")
        self.chk_cli_tree.pack(anchor="w", pady=2)

        ctk.CTkLabel(tab_cli, text="Дефолтный промпт для CLI:", anchor="w").pack(fill="x", pady=(10, 0))
        self.txt_cli_prompt = ctk.CTkTextbox(tab_cli, height=100)
        self.txt_cli_prompt.pack(fill="x", pady=5)

        ctk.CTkButton(tab_cli, text="💾 Сохранить настройки CLI", fg_color="green", command=self.save_cli_settings).pack(fill="x", pady=20)

        # Load Defaults into CLI Tab
        self.load_cli_ui()

        # --- Main Area ---
        main = ctk.CTkFrame(self, fg_color="transparent")
        main.grid(row=0, column=1, sticky="nsew", padx=20, pady=20)
        main.grid_rowconfigure(3, weight=1)
        main.grid_columnconfigure(0, weight=1)

        # Folders List
        self.scroll_folders = ctk.CTkScrollableFrame(main, height=100, label_text="Выбранные источники")
        self.scroll_folders.grid(row=0, column=0, sticky="ew", pady=(0, 10))

        # Current Run Toggles
        opts_frame = ctk.CTkFrame(main, fg_color="transparent")
        opts_frame.grid(row=1, column=0, sticky="ew", pady=(0, 10))

        self.chk_minify = ctk.CTkCheckBox(opts_frame, text="Minify")
        self.chk_minify.pack(side="left", padx=10)
        self.chk_comments = ctk.CTkCheckBox(opts_frame, text="No Comments")
        self.chk_comments.pack(side="left", padx=10)
        self.chk_secrets = ctk.CTkCheckBox(opts_frame, text="No Secrets")
        self.chk_secrets.select()
        self.chk_secrets.pack(side="left", padx=10)

        ctk.CTkLabel(opts_frame, text="Формат:").pack(side="left", padx=(20, 5))
        self.format_var = ctk.StringVar(value="Markdown")
        ctk.CTkSegmentedButton(opts_frame, values=["Markdown", "XML", "Plain"], variable=self.format_var).pack(side="left")

        # Buttons
        btn_frame = ctk.CTkFrame(main)
        btn_frame.grid(row=2, column=0, sticky="ew", pady=(0, 10))

        self.btn_clip = ctk.CTkButton(btn_frame, text="📋 В Буфер", height=40, font=ctk.CTkFont(weight="bold"),
                                      command=lambda: self.run_task('clipboard'))
        self.btn_clip.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        self.btn_file = ctk.CTkButton(btn_frame, text="💾 В Файл", height=40, command=lambda: self.run_task('file'))
        self.btn_file.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        self.btn_pdf = ctk.CTkButton(btn_frame, text="📄 В PDF", height=40, command=lambda: self.run_task('pdf'))
        self.btn_pdf.pack(side="left", padx=10, pady=10, expand=True, fill="x")

        # Logs
        self.log_box = ctk.CTkTextbox(main, font=("Consolas", 12))
        self.log_box.grid(row=3, column=0, sticky="nsew")
        self.log_box.insert("0.0", "Готов к работе.\n")

        # Status
        self.status_frame = ctk.CTkFrame(main, height=30)
        self.status_frame.grid(row=4, column=0, sticky="ew", pady=(10, 0))
        self.progress = ctk.CTkProgressBar(self.status_frame)
        self.progress.pack(fill="x", side="top")
        self.progress.set(0)
        self.lbl_stat = ctk.CTkLabel(self.status_frame, text="Idle", text_color="gray")
        self.lbl_stat.pack(side="left", padx=5)
        self.lbl_token = ctk.CTkLabel(self.status_frame, text="Tokens: 0", text_color="gray")
        self.lbl_token.pack(side="right", padx=5)

    def load_cli_ui(self):
        """Load settings from JSON to CLI Tab widgets."""
        cfg = SettingsManager.load()
        if cfg['cli_minify']: self.chk_cli_minify.select()
        else: self.chk_cli_minify.deselect()

        if cfg['cli_remove_comments']: self.chk_cli_comments.select()
        else: self.chk_cli_comments.deselect()

        if cfg['cli_remove_secrets']: self.chk_cli_secrets.select()
        else: self.chk_cli_secrets.deselect()

        if cfg['cli_include_tree']: self.chk_cli_tree.select()
        else: self.chk_cli_tree.deselect()

        self.txt_cli_prompt.delete("0.0", "end")
        self.txt_cli_prompt.insert("0.0", cfg['cli_system_prompt'])

    def save_cli_settings(self):
        """Save UI CLI settings to JSON."""
        settings = SettingsManager.load() # Get base to keep exts/igns if needed
        settings.update({
            "cli_minify": bool(self.chk_cli_minify.get()),
            "cli_remove_comments": bool(self.chk_cli_comments.get()),
            "cli_remove_secrets": bool(self.chk_cli_secrets.get()),
            "cli_include_tree": bool(self.chk_cli_tree.get()),
            "cli_system_prompt": self.txt_cli_prompt.get("1.0", "end-1c"),
            # We can also save current extensions as CLI defaults if we want
            "cli_exts": self.entry_ext.get(),
            "cli_ign": self.entry_ign.get()
        })
        SettingsManager.save(settings)
        messagebox.showinfo("Настройки", "Настройки для контекстного меню сохранены!")

    def apply_preset(self, choice):
        data = PRESETS.get(choice)
        if data:
            self.entry_ext.delete(0, "end")
            self.entry_ext.insert(0, data["ext"])
            self.entry_ign.delete(0, "end")
            self.entry_ign.insert(0, data["ign"])

    def add_folder(self):
        path = filedialog.askdirectory()
        if path and path not in self.selected_folders:
            self.selected_folders.append(path)
            row = ctk.CTkFrame(self.scroll_folders)
            row.pack(fill="x", pady=2)
            ctk.CTkLabel(row, text=f"📂 {path}", anchor="w").pack(side="left", padx=5)

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

    def run_task(self, target):
        if not self.selected_folders:
            messagebox.showwarning("Ошибка", "Нет папок для обработки")
            return

        self.lock_ui(True)
        self.log_box.configure(state="normal")
        self.log_box.delete("1.0", "end")
        self.log_box.configure(state="disabled")

        params = {
            'exts': self.entry_ext.get().split(),
            'ign': {x.strip() for x in self.entry_ign.get().split(',')},
            'minify': bool(self.chk_minify.get()),
            'remove_comments': bool(self.chk_comments.get()),
            'remove_secrets': bool(self.chk_secrets.get()),
            'use_git': bool(self.chk_git.get()),
            'include_tree': bool(self.chk_tree.get()),
            'system_prompt': self.txt_prompt.get("1.0", "end-1c"),
            'format': self.format_var.get().lower(),
            'target': target
        }

        threading.Thread(target=self._worker, args=(params,), daemon=True).start()

    def _worker(self, params):
        try:
            self.update_status("Сканирование...", 0.1)
            scanner = FileScanner(params['exts'], params['ign'])
            files_to_process = scanner.scan(self.selected_folders, use_git=params['use_git'])

            total = len(files_to_process)
            if total == 0:
                self.log("❌ Файлы не найдены.")
                self.update_status("Нет файлов", 0)
                return

            self.log(f"🔎 Найдено файлов: {total}")
            file_entries = []

            for i, path in enumerate(files_to_process):
                try:
                    if path.stat().st_size > MAX_FILE_SIZE_MB * 1024 * 1024:
                        continue
                    content = path.read_text(encoding='utf-8', errors='replace')
                    clean_content = self.cleaner_service.process(content, path.suffix, params)
                    file_entries.append({'path': str(path), 'content': clean_content})
                except Exception: pass

                if i % 10 == 0:
                    self.update_status(f"Обработка...", 0.2 + (0.6 * (i / total)))

            self.update_status("Форматирование...", 0.9)
            result_text = FormatterService.format(
                file_entries,
                params['format'],
                include_tree=params['include_tree'],
                system_prompt=params['system_prompt']
            )

            tokens = self.token_service.count(result_text)
            self.lbl_token.configure(text=f"Tokens: ~{tokens}")

            target = params['target']
            if target == 'clipboard':
                pyperclip.copy(result_text)
                self.log(f"✅ Успешно! Скопировано.")
            elif target == 'file':
                ext = f".{params['format']}" if params['format'] != 'plain' else ".txt"
                path = filedialog.asksaveasfilename(defaultextension=ext)
                if path:
                    Path(path).write_text(result_text, encoding='utf-8')
                    self.log(f"✅ Сохранено: {path}")
            elif target == 'pdf':
                path = filedialog.asksaveasfilename(defaultextension=".pdf")
                if path:
                    PdfService.create_pdf(result_text, path)
                    self.log(f"✅ PDF создан: {path}")

            self.update_status("Готово", 1.0)
        except Exception as e:
            self.log(f"🔥 Ошибка: {e}")
            import traceback
            traceback.print_exc()
        finally:
            self.lock_ui(False)