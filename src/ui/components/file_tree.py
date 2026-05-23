import tkinter as tk
from tkinter import ttk
import customtkinter as ctk
import os
import threading
from typing import List, Callable, Tuple, Any


class FileTree(ctk.CTkFrame):
    def __init__(self, parent: Any, on_toggle_callback: Callable[[str, bool], None],
                 on_context_action_callback: Callable[[str, bool], None]):
        super().__init__(parent, fg_color="transparent")
        self.on_toggle = on_toggle_callback
        self.on_context_action = on_context_action_callback
        self.file_paths: List[str] = []
        self._lock = threading.Lock()
        self._current_task_id = 0

        self._init_ui()
        self._setup_style()

    def _init_ui(self):
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # --- Empty State Label ---
        self.empty_label = ctk.CTkLabel(
            self,
            text="📂 Перетащите папки или файлы в окно,\nчтобы увидеть структуру",
            text_color="gray",
            font=("Arial", 14)
        )
        self.empty_label.grid(row=0, column=0, sticky="nsew")
        # -------------------------

        self.loading_label = ctk.CTkLabel(self, text="Построение дерева...", text_color="gray")
        self.tree = ttk.Treeview(self, show="tree", selectmode="none", style="CodeContext.Treeview")
        self.vsb = ctk.CTkScrollbar(self, orientation="vertical", command=self.tree.yview)
        self.hsb = ctk.CTkScrollbar(self, orientation="horizontal", command=self.tree.xview)

        self.tree.configure(yscrollcommand=self.vsb.set, xscrollcommand=self.hsb.set)

        # Tree is initially hidden (shown only when populated)
        # self.tree.grid(row=0, column=0, sticky="nsew")

        self.vsb.grid(row=0, column=1, sticky="ns")
        self.hsb.grid(row=1, column=0, sticky="ew")

        self.tree.bind("<Button-1>", self._on_click)
        self.tree.bind("<Button-3>", self._on_right_click)

    @staticmethod
    def _setup_style():
        style = ttk.Style()
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
        if not file_paths:
            self.delete_all()
            return

        with self._lock:
            self._current_task_id += 1
            task_id = self._current_task_id
            self.file_paths = file_paths

            # Hide tree and empty state, show loading
            self.tree.grid_forget()
            self.empty_label.grid_forget()
            self.loading_label.grid(row=0, column=0, sticky="nsew")

            threading.Thread(
                target=self._prepare_nodes_thread,
                args=(file_paths, task_id),
                daemon=True
            ).start()

    def _prepare_nodes_thread(self, paths: List[str], task_id: int):
        try:
            paths = sorted(paths)
            try:
                common = os.path.commonpath(paths)
                if os.path.isfile(common):
                    common = os.path.dirname(common)
            except ValueError:
                common = ""

            nodes_to_insert = []
            created_ids = set()

            for full_path in paths:
                if task_id != self._current_task_id:
                    return
                try:
                    rel_path = os.path.relpath(full_path, common)
                except ValueError:
                    rel_path = full_path

                parts = rel_path.split(os.sep)
                current_id = ""

                for i, part in enumerate(parts):
                    parent_id = current_id
                    if common:
                        current_id = os.path.join(common, *parts[:i + 1])
                    else:
                        current_id = os.path.join(*parts[:i + 1])

                    if current_id not in created_ids:
                        text = f"☑ {part}"
                        nodes_to_insert.append((parent_id, current_id, text))
                        created_ids.add(current_id)

            if task_id == self._current_task_id:
                self.after(0, self._bulk_insert_ui, nodes_to_insert, task_id)
        except Exception as e:
            print(f"Error in tree thread: {e}")
            # Revert to empty state on error if needed, or just hide loading
            self.after(0, self.delete_all)

    def _bulk_insert_ui(self, nodes: List[Tuple], task_id: int):
        if task_id != self._current_task_id:
            return

        # Sctrictly hide empty state when filling
        self.empty_label.grid_forget()

        self.delete_all_tree_only()  # Clear existing items without changing grid layout yet

        for parent, iid, text in nodes:
            try:
                self.tree.insert(parent, "end", iid, text=text, open=True)
            except tk.TclError:
                pass

        # Hide loading, show tree
        self.loading_label.grid_forget()
        self.tree.grid(row=0, column=0, sticky="nsew")

    def delete_all_tree_only(self):
        """Clears tree items but keeps the widget visible/grid configuration intact."""
        self.tree.delete(*self.tree.get_children())

    def delete_all(self):
        """Resets the component to the initial Empty State."""
        self.delete_all_tree_only()
        self.tree.grid_forget()
        self.loading_label.grid_forget()
        self.empty_label.grid(row=0, column=0, sticky="nsew")

    def _on_click(self, event: Any):
        region = self.tree.identify("region", event.x, event.y)
        if region != "tree":
            return
        item_id = self.tree.identify_row(event.y)
        if not item_id:
            return

        item_text = self.tree.item(item_id, "text")

        if item_text.startswith("☑"):
            new_text = item_text.replace("☑", "☐", 1)
            new_state = False
        elif item_text.startswith("☐"):
            new_text = item_text.replace("☐", "☑", 1)
            new_state = True
        else:
            return

        self.tree.item(item_id, text=new_text)
        self._propagate_check(item_id, new_state)
        self._update_controller_state(item_id, new_state)

    def _on_right_click(self, event: Any):
        item_id = self.tree.identify_row(event.y)
        if not item_id or not os.path.isfile(item_id):
            return

        self.tree.selection_set(item_id)
        menu = tk.Menu(self, tearoff=0)
        menu.add_command(label="Копировать с зависимостями (Shallow)",
                         command=lambda: self.on_context_action(item_id, False))
        menu.add_command(label="Копировать с зависимостями (Deep)",
                         command=lambda: self.on_context_action(item_id, True))
        menu.post(event.x_root, event.y_root)

    def _propagate_check(self, item_id: str, state: bool):
        children = self.tree.get_children(item_id)
        icon = "☑" if state else "☐"
        stack = list(children)
        while stack:
            child = stack.pop()
            old_text = self.tree.item(child, "text")
            if isinstance(old_text, str) and len(old_text) > 2:
                clean_name = old_text[2:]
                self.tree.item(child, text=f"{icon} {clean_name}")
                self._update_controller_state(str(child), state)
            stack.extend(self.tree.get_children(child))

    def _update_controller_state(self, item_id: str, state: bool):
        if os.path.isfile(item_id):
            self.on_toggle(item_id, state)