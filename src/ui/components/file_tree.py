import tkinter as tk
from tkinter import ttk
import customtkinter as ctk
import os
import threading
from typing import List, Callable, Tuple


class FileTree(ctk.CTkFrame):
    def __init__(self, parent, on_toggle_callback: Callable[[str, bool], None]):
        super().__init__(parent, fg_color="transparent")
        self.on_toggle = on_toggle_callback
        self.file_paths = []

        # Блокировка для защиты от гонки потоков при быстрых переключениях
        self._lock = threading.Lock()
        self._current_task_id = 0

        self._init_ui()
        self._setup_style()

    def _init_ui(self):
        # Сетка
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # Индикатор загрузки (скрыт по умолчанию)
        self.loading_label = ctk.CTkLabel(self, text="Построение дерева...", text_color="gray")

        # Основной виджет дерева
        self.tree = ttk.Treeview(self, show="tree", selectmode="none", style="CodeContext.Treeview")

        # Скроллбары
        self.vsb = ctk.CTkScrollbar(self, orientation="vertical", command=self.tree.yview)
        self.hsb = ctk.CTkScrollbar(self, orientation="horizontal", command=self.tree.xview)
        self.tree.configure(yscrollcommand=self.vsb.set, xscrollcommand=self.hsb.set)

        # Размещение (изначально дерево видно)
        self.tree.grid(row=0, column=0, sticky="nsew")
        self.vsb.grid(row=0, column=1, sticky="ns")
        self.hsb.grid(row=1, column=0, sticky="ew")

        # Биндинг
        self.tree.bind("<Button-1>", self._on_click)

    def _setup_style(self):
        style = ttk.Style()

        # Цвета под темную тему
        bg_color = "#2b2b2b"
        fg_color = "#dce4ee"
        sel_bg = "#1f538d"

        style.theme_use("default")
        style.configure("CodeContext.Treeview",
                        background=bg_color,
                        foreground=fg_color,
                        fieldbackground=bg_color,
                        borderwidth=0,
                        font=("Consolas", 10))

        style.map("CodeContext.Treeview",
                  background=[("selected", sel_bg)],
                  foreground=[("selected", "white")])

    def populate(self, file_paths: List[str]):
        """
        Публичный метод обновления дерева.
        Запускает фоновый поток для подготовки данных.
        """
        if not file_paths:
            self.delete_all()
            return

        # Увеличиваем ID задачи, чтобы старые потоки (если они еще бегут) знали, что их результат не нужен
        with self._lock:
            self._current_task_id += 1
            task_id = self._current_task_id

        self.file_paths = file_paths

        # Показываем "Загрузку", скрываем дерево (чтобы не мигало при очистке)
        self.tree.grid_remove()
        self.loading_label.grid(row=0, column=0, sticky="nsew")

        # Запускаем тяжелую логику в потоке
        threading.Thread(
            target=self._prepare_nodes_thread,
            args=(file_paths, task_id),
            daemon=True
        ).start()

    def _prepare_nodes_thread(self, paths: List[str], task_id: int):
        """
        Выполняется в фоновом потоке.
        Готовит плоский список операций вставки, чтобы не делать логику в GUI потоке.
        """
        try:
            # 1. Сортировка (тяжелая операция для тысяч файлов)
            paths = sorted(paths)

            # 2. Поиск корня
            try:
                common = os.path.commonpath(paths)
                if os.path.isfile(common):
                    common = os.path.dirname(common)
            except ValueError:
                common = ""

            # 3. Генерация данных для вставки
            # Список кортежей: (parent_id, current_id, text_to_display, is_file)
            nodes_to_insert = []
            created_ids = set()  # Локальный кэш для скорости (быстрее, чем tree.exists)

            for full_path in paths:
                # Если задача устарела (пользователь уже запустил новое сканирование), останавливаемся
                if task_id != self._current_task_id:
                    return

                try:
                    rel_path = os.path.relpath(full_path, common)
                except ValueError:
                    rel_path = full_path

                parts = rel_path.split(os.sep)
                current_id = ""

                # Проход по частям пути
                for i, part in enumerate(parts):
                    parent_id = current_id

                    # Формируем уникальный ID.
                    # Важно: используем абсолютные пути или части, чтобы ID были уникальны.
                    # Здесь используем накопление частей пути.
                    if common:
                        current_id = os.path.join(common, *parts[:i + 1])
                    else:
                        current_id = os.path.join(*parts[:i + 1])

                    if current_id not in created_ids:
                        text = f"☑ {part}"
                        is_file = (i == len(parts) - 1)

                        # Добавляем в список "на вставку"
                        nodes_to_insert.append((parent_id, current_id, text))
                        created_ids.add(current_id)

            # 4. Передача данных в GUI поток
            if task_id == self._current_task_id:
                self.after(0, lambda: self._bulk_insert_ui(nodes_to_insert, task_id))

        except Exception as e:
            print(f"Error in tree thread: {e}")

    def _bulk_insert_ui(self, nodes: List[Tuple], task_id: int):
        """
        Выполняется в главном потоке.
        Быстро вставляет подготовленные ноды.
        """
        if task_id != self._current_task_id:
            return

        # Очистка
        self.delete_all()

        # Массовая вставка
        # tree.grid_remove() уже вызван в populate, поэтому перерисовки не будет
        for parent, iid, text in nodes:
            # parent="" для корня
            try:
                self.tree.insert(parent, "end", iid, text=text, open=True)
            except tk.TclError:
                pass  # Игнорируем дубликаты или ошибки путей

        # Возвращаем интерфейс
        self.loading_label.grid_forget()
        self.tree.grid(row=0, column=0, sticky="nsew")

    def delete_all(self):
        """Очистка дерева"""
        self.tree.delete(*self.tree.get_children())

    def _on_click(self, event):
        """Обработка клика (без изменений логики, только оптимизация)"""
        # Определяем, куда кликнули
        region = self.tree.identify("region", event.x, event.y)
        if region != "tree":
            return

        item_id = self.tree.identify_row(event.y)
        if not item_id:
            return

        # Получаем текст
        item_text = self.tree.item(item_id, "text")

        # Переключаем иконку
        new_state = False
        if item_text.startswith("☑"):
            new_text = item_text.replace("☑", "☐", 1)
            new_state = False
        elif item_text.startswith("☐"):
            new_text = item_text.replace("☐", "☑", 1)
            new_state = True
        else:
            return

            # Обновляем этот элемент
        self.tree.item(item_id, text=new_text)

        # Запускаем рекурсивное обновление детей (можно в потоке, если папка огромная,
        # но обычно это быстро для видимой части)
        self._propagate_check(item_id, new_state)

        # Обновляем состояние в контроллере
        self._update_controller_state(item_id, new_state)

    def _propagate_check(self, item_id, state):
        """Рекурсивно ставит галочки детям"""
        children = self.tree.get_children(item_id)
        icon = "☑" if state else "☐"

        # Используем стек вместо рекурсии для безопасности при огромной вложенности
        stack = list(children)
        while stack:
            child = stack.pop()
            old_text = self.tree.item(child, "text")
            if len(old_text) > 2:
                clean_name = old_text[2:]
                self.tree.item(child, text=f"{icon} {clean_name}")
                self._update_controller_state(child, state)

            # Добавляем детей в стек
            stack.extend(self.tree.get_children(child))

    def _update_controller_state(self, item_id, state):
        """Вызов коллбека"""
        if os.path.isfile(item_id):
            self.on_toggle(item_id, state)