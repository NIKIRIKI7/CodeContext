import tkinter as tk
from tkinter import ttk
import customtkinter as ctk
import os
import threading
from typing import List, Callable, Tuple, Any


class FileTree(ctk.CTkFrame):
    def __init__(self, parent: Any, on_toggle_callback: Callable, on_context_action_callback: Callable):
        super().__init__(parent, fg_color="transparent")
        self.on_toggle = on_toggle_callback
        self.on_context_action = on_context_action_callback
        self.file_paths: List[str] = []
        self.metadata: dict = {}
        self._lock = threading.Lock()
        self._current_task_id = 0
        self._init_ui()

    def _init_ui(self):
        self.grid_columnconfigure(0, weight=1)
        self.grid_rowconfigure(1, weight=1)

        top_frame = ctk.CTkFrame(self, fg_color="transparent")
        top_frame.grid(row=0, column=0, sticky="ew", pady=(0, 5))

        self.entry_search = ctk.CTkEntry(top_frame, placeholder_text="🔍 Поиск файлов...")
        self.entry_search.pack(side="left", fill="x", expand=True, padx=(0, 5))
        self.entry_search.bind("<KeyRelease>", self._on_search)

        self.btn_select_filtered = ctk.CTkButton(top_frame, text="☑ Выбрать эти", width=100,
                                                 command=self._select_filtered)
        self.btn_select_filtered.pack(side="right")

        self.empty_label = ctk.CTkLabel(
            self,
            text="📂 Перетащите папки или файлы в окно,\nчтобы увидеть структуру",
            text_color="gray",
            font=("Arial", 14)
        )
        self.empty_label.grid(row=1, column=0, sticky="nsew")

        self.loading_label = ctk.CTkLabel(self, text="Построение дерева...", text_color="gray")
        self.tree = ttk.Treeview(self, show="tree", selectmode="none")
        self.vsb = ctk.CTkScrollbar(self, orientation="vertical", command=self.tree.yview)
        self.hsb = ctk.CTkScrollbar(self, orientation="horizontal", command=self.tree.xview)
        self.tree.configure(yscrollcommand=self.vsb.set, xscrollcommand=self.hsb.set)

        self.vsb.grid(row=1, column=1, sticky="ns")
        self.hsb.grid(row=2, column=0, sticky="ew")

        self.tree.tag_configure("added", foreground="#6bc46d")
        self.tree.tag_configure("modified", foreground="#d4b85c")

        self.tree.bind("<Button-1>", self._on_click)
        self.tree.bind("<Button-3>", self._on_right_click)

    def populate(self, file_paths: List[str], metadata: dict):
        if not file_paths:
            self.delete_all()
            return
        with self._lock:
            self._current_task_id += 1
            self.file_paths = file_paths
            self.metadata = metadata
            self.empty_label.grid_forget()
            self.tree.grid_remove()
            self.loading_label.grid(row=1, column=0, sticky="nsew")
            threading.Thread(target=self._prepare_nodes_thread, args=(file_paths, self._current_task_id),
                             daemon=True).start()

    def _prepare_nodes_thread(self, paths: List[str], task_id: int):
        try:
            paths = sorted(paths)
            search_query = self.entry_search.get().lower()

            common = os.path.commonpath(paths) if paths else ""
            if common and os.path.isfile(common):
                common = os.path.dirname(common)

            nodes_to_insert = []
            created_ids = set()

            for full_path in paths:
                if search_query and search_query not in full_path.lower():
                    continue

                rel_path = os.path.relpath(full_path, common) if common else full_path
                parts = rel_path.split(os.sep)
                current_id = ""

                for i, part in enumerate(parts):
                    parent_id = current_id
                    current_id = os.path.join(common, *parts[:i + 1]) if common else os.path.join(*parts[:i + 1])

                    if current_id not in created_ids:
                        text = part
                        tags = ()

                        if current_id == full_path:
                            meta = self.metadata.get(full_path, {})
                            tokens = meta.get("tokens", 0)
                            git_status = meta.get("git_status", "")

                            token_str = f"({tokens / 1000:.1f}k tk)" if tokens > 1000 else f"({tokens} tk)"
                            text = f"☑ {part} {token_str}"
                            if git_status:
                                tags = (git_status,)
                        else:
                            text = f"☑ {part}"

                        nodes_to_insert.append((parent_id, current_id, text, tags))
                        created_ids.add(current_id)

            if task_id == self._current_task_id:
                self.after(0, self._bulk_insert_ui, nodes_to_insert, task_id)
        except Exception as e:
            print(f"Error in tree thread: {e}")

    def _bulk_insert_ui(self, nodes: List[Tuple], task_id: int):
        if task_id != self._current_task_id: return
        self.delete_all_tree_only()
        for parent, iid, text, tags in nodes:
            try:
                self.tree.insert(parent, "end", iid, text=text, tags=tags, open=True)
            except tk.TclError:
                pass
        self.loading_label.grid_forget()
        self.tree.grid(row=1, column=0, sticky="nsew")

    def delete_all_tree_only(self):
        self.tree.delete(*self.tree.get_children())

    def delete_all(self):
        self.delete_all_tree_only()
        self.tree.grid_forget()
        self.empty_label.grid(row=1, column=0, sticky="nsew")

    def _on_search(self, event):
        self.populate(self.file_paths, self.metadata)

    def _select_filtered(self):
        search_query = self.entry_search.get().lower()
        if not search_query: return
        for path in self.file_paths:
            is_match = search_query in path.lower()
            self.on_toggle(path, is_match)
        self.populate(self.file_paths, self.metadata)

    def _on_click(self, event: Any):
        region = self.tree.identify("region", event.x, event.y)
        if region != "tree": return
        item_id = self.tree.identify_row(event.y)
        if not item_id: return

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