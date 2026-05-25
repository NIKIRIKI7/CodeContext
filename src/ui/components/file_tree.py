import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QTreeView, QLineEdit, QMenu
from PySide6.QtGui import QStandardItemModel, QStandardItem
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus


class FileTree(QWidget):
    def __init__(self, on_toggle_callback, on_context_action_callback):
        super().__init__()
        self.on_toggle = on_toggle_callback
        self.on_context = on_context_action_callback

        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)

        self.search = QLineEdit()
        self.search.setPlaceholderText("🔍 Поиск файлов...")

        self.tree = QTreeView()
        self.tree.setHeaderHidden(True)
        self.tree.setContextMenuPolicy(Qt.CustomContextMenu)
        self.tree.customContextMenuRequested.connect(self._show_context_menu)

        self.model = QStandardItemModel()
        self.tree.setModel(self.model)
        self.model.itemChanged.connect(self._on_item_changed)

        self.layout.addWidget(self.search)
        self.layout.addWidget(self.tree)

        self.search.textChanged.connect(self._filter_tree)
        self._all_items = []
        self._is_updating = False

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setSpacing(s)

    def populate(self, file_paths, metadata, manual_exclusions):
        if self._is_updating:
            return

        self._is_updating = True
        self.model.clear()
        self._all_items.clear()

        root = self.model.invisibleRootItem()
        common = os.path.commonpath(file_paths) if file_paths else ""
        if common and os.path.isfile(common):
            common = os.path.dirname(common)

        node_map = {"": root}

        # 💡 UI просто мапит категории на иконки, не зная логики их определения
        icon_map = {
            "DEPENDENCY": "📦 [ЗАВИСИМОСТЬ]",
            "HUGE": "🔥 [ОГРОМНЫЙ]",
            "HEAVY": "🔴 [ТЯЖЕЛЫЙ]",
            "MEDIUM": "🟡",
            "LIGHT": "🟢"
        }

        for path in sorted(file_paths):
            rel_path = os.path.relpath(path, common) if common else path
            parts = rel_path.split(os.sep)
            curr = ""
            for i, part in enumerate(parts):
                parent_key = curr
                curr = os.path.join(curr, part) if curr else part

                if curr not in node_map:
                    item = QStandardItem()
                    item.setCheckable(True)

                    full_path = os.path.join(common, curr) if common else curr
                    is_file = (i == len(parts) - 1)

                    item.setData(full_path, Qt.UserRole)
                    item.setData(is_file, Qt.UserRole + 1)

                    if is_file:
                        meta = metadata.get(full_path, {})
                        tokens = meta.get("tokens", 0)
                        category = meta.get("category", "LIGHT")

                        item.setData(tokens, Qt.UserRole + 2)
                        item.setData(category, Qt.UserRole + 3)

                        icon = icon_map.get(category, "🟢")
                        item.setText(f"{icon} {part} ({tokens} tk)")

                        state = Qt.Unchecked if full_path in manual_exclusions else Qt.Checked
                        item.setCheckState(state)
                        self._all_items.append((item, full_path))
                    else:
                        item.setText(f"📁 {part}")
                        item.setCheckState(Qt.Checked)

                    node_map[parent_key].appendRow(item)
                    node_map[curr] = item

        self.tree.expandAll()
        self._is_updating = False

    def _on_item_changed(self, item):
        if self._is_updating:
            return

        state = item.checkState() == Qt.Checked
        path = item.data(Qt.UserRole)
        is_file = item.data(Qt.UserRole + 1)

        if path and is_file:
            self._is_updating = True
            self.on_toggle(path, state)
            self._is_updating = False
        else:
            self._propagate_check(item, state)

    def _propagate_check(self, parent_item, state):
        self._is_updating = True
        for row in range(parent_item.rowCount()):
            child = parent_item.child(row)
            child.setCheckState(Qt.Checked if state else Qt.Unchecked)
            path = child.data(Qt.UserRole)
            is_file = child.data(Qt.UserRole + 1)

            if path and is_file:
                self.on_toggle(path, state)

            if child.hasChildren():
                self._propagate_check_recursive(child, state)
        self._is_updating = False

    def _propagate_check_recursive(self, parent_item, state):
        for row in range(parent_item.rowCount()):
            child = parent_item.child(row)
            child.setCheckState(Qt.Checked if state else Qt.Unchecked)
            path = child.data(Qt.UserRole)
            is_file = child.data(Qt.UserRole + 1)

            if path and is_file:
                self.on_toggle(path, state)

            if child.hasChildren():
                self._propagate_check_recursive(child, state)

    def _filter_tree(self, text):
        search_query = text.lower()
        for item, full_path in self._all_items:
            match = search_query in full_path.lower()
            parent = item.parent()
            root = self.model.invisibleRootItem()
            self.tree.setRowHidden(item.row(), parent.index() if parent else root.index(), not match)

    def _show_context_menu(self, position):
        index = self.tree.indexAt(position)
        if not index.isValid(): return

        item = self.model.itemFromIndex(index)
        path = item.data(Qt.UserRole)
        is_file = item.data(Qt.UserRole + 1)

        if not path: return

        menu = QMenu()

        if is_file:
            act_shallow = menu.addAction("Копировать с зависимостями (Shallow)")
            act_deep = menu.addAction("Копировать с зависимостями (Deep)")
            action = menu.exec(self.tree.viewport().mapToGlobal(position))

            if action == act_shallow:
                self.on_context(path, False)
            elif action == act_deep:
                self.on_context(path, True)
        else:
            act_only_py = menu.addAction("Выбрать только .py файлы внутри")
            act_only_js = menu.addAction("Выбрать только .js / .ts / .vue внутри")
            act_exclude_heavy = menu.addAction("Исключить зависимости и тяжелые файлы")
            menu.addSeparator()
            act_exclude_all = menu.addAction("Исключить все файлы внутри")

            action = menu.exec(self.tree.viewport().mapToGlobal(position))

            if action == act_only_py:
                self._filter_children_by_ext(item, ['.py'])
            elif action == act_only_js:
                self._filter_children_by_ext(item, ['.js', '.ts', '.jsx', '.tsx', '.vue'])
            elif action == act_exclude_heavy:
                self._exclude_heavy_children(item)
            elif action == act_exclude_all:
                self._exclude_all_children(item)

    def _filter_children_by_ext(self, parent_item, exts):
        self._is_updating = True
        self._apply_ext_filter_recursive(parent_item, exts)
        self._is_updating = False

    def _apply_ext_filter_recursive(self, parent_item, exts):
        for row in range(parent_item.rowCount()):
            child = parent_item.child(row)
            is_file = child.data(Qt.UserRole + 1)
            path = child.data(Qt.UserRole)

            if is_file and path:
                ext = os.path.splitext(path)[1].lower()
                state = Qt.Checked if ext in exts else Qt.Unchecked
                child.setCheckState(state)
                self.on_toggle(path, state == Qt.Checked)
            elif child.hasChildren():
                self._apply_ext_filter_recursive(child, exts)

    def _exclude_heavy_children(self, parent_item):
        """Снимает галочки с файлов, отмеченных как DEPENDENCY, HUGE или HEAVY"""
        self._is_updating = True
        self._apply_heavy_filter_recursive(parent_item)
        self._is_updating = False

    def _apply_heavy_filter_recursive(self, parent_item):
        for row in range(parent_item.rowCount()):
            child = parent_item.child(row)
            is_file = child.data(Qt.UserRole + 1)

            if is_file:
                category = child.data(Qt.UserRole + 3)
                if category in ("HUGE", "HEAVY", "DEPENDENCY"):
                    child.setCheckState(Qt.Unchecked)
                    self.on_toggle(child.data(Qt.UserRole), False)
            elif child.hasChildren():
                self._apply_heavy_filter_recursive(child)

    def _exclude_all_children(self, parent_item):
        self._is_updating = True
        self._propagate_check_recursive(parent_item, False)
        parent_item.setCheckState(Qt.Unchecked)
        self._is_updating = False

    def clear(self):
        self._is_updating = True
        self.model.clear()
        self._all_items.clear()
        self._is_updating = False