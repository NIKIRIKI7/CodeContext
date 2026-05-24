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

        # Флаг-защитник от рекурсии и крашей при очистке памяти
        self._is_updating = False

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setSpacing(s)

    def populate(self, file_paths, metadata, manual_exclusions):
        # БЛОКИРУЕМ ПЕРЕРИСОВКУ, ЕСЛИ МЫ СЕЙЧАС КЛИКАЕМ ПО ЧЕКБОКСАМ
        # Это полностью устраняет краш от удаления C++ объектов во время их перебора
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

        for path in sorted(file_paths):
            rel_path = os.path.relpath(path, common) if common else path
            parts = rel_path.split(os.sep)
            curr = ""

            for i, part in enumerate(parts):
                parent_key = curr
                curr = os.path.join(curr, part) if curr else part

                if curr not in node_map:
                    item = QStandardItem(part)
                    item.setCheckable(True)

                    full_path = os.path.join(common, curr) if common else curr
                    is_file = (i == len(parts) - 1)

                    if is_file:
                        meta = metadata.get(full_path, {})
                        tokens = meta.get("tokens", 0)
                        item.setText(f"{part} ({tokens} tk)")
                        item.setData(full_path, Qt.UserRole)
                        state = Qt.Unchecked if full_path in manual_exclusions else Qt.Checked
                        item.setCheckState(state)
                        self._all_items.append((item, full_path))
                    else:
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

        if path:
            # Защита для одиночного клика по файлу
            self._is_updating = True
            self.on_toggle(path, state)
            self._is_updating = False
        else:
            # Клик по папке: каскадное обновление файлов
            self._propagate_check(item, state)

    def _propagate_check(self, parent_item, state):
        self._is_updating = True

        for row in range(parent_item.rowCount()):
            child = parent_item.child(row)
            # Изменение состояния вызовет _on_item_changed, но он сразу вернет return
            child.setCheckState(Qt.Checked if state else Qt.Unchecked)

            path = child.data(Qt.UserRole)
            if path:
                self.on_toggle(path, state)

            if child.hasChildren():
                self._propagate_check_recursive(child, state)

        self._is_updating = False

    def _propagate_check_recursive(self, parent_item, state):
        for row in range(parent_item.rowCount()):
            child = parent_item.child(row)
            child.setCheckState(Qt.Checked if state else Qt.Unchecked)

            path = child.data(Qt.UserRole)
            if path:
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
        if not path: return

        menu = QMenu()
        act_shallow = menu.addAction("Копировать с зависимостями (Shallow)")
        act_deep = menu.addAction("Копировать с зависимостями (Deep)")

        action = menu.exec(self.tree.viewport().mapToGlobal(position))
        if action == act_shallow:
            self.on_context(path, False)
        elif action == act_deep:
            self.on_context(path, True)

    def clear(self):
        self._is_updating = True
        self.model.clear()
        self._all_items.clear()
        self._is_updating = False