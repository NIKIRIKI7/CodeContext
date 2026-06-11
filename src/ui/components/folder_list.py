import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QScrollArea, QFrame
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus
from src.i18n import tr


class FolderList(QScrollArea):
    def __init__(self, on_edit, on_delete):
        super().__init__()
        self.on_edit = on_edit
        self.on_delete = on_delete
        self.setWidgetResizable(True)
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setProperty("cssClass", "card")

        self.container = QWidget()
        self.container.setObjectName("folder_container")
        self.layout = QVBoxLayout(self.container)
        self.layout.setAlignment(Qt.AlignTop)
        self.setWidget(self.container)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        s = ThemeManager.get_layout("panel_spacing", 16)
        self.layout.setContentsMargins(m, m, m, m)
        self.layout.setSpacing(int(s / 2))

    def update_ui(self, selected_folders, temp_folders):
        while self.layout.count():
            item = self.layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()

        if not selected_folders:
            lbl = QLabel(tr("folder_list.no_sources"))
            lbl.setProperty("cssClass", "muted")
            self.layout.addWidget(lbl)
            return

        for folder in selected_folders:
            row = QFrame()
            row.setProperty("cssClass", "recent_card")
            row_layout = QHBoxLayout(row)
            row_layout.setContentsMargins(12, 8, 12, 8)

            prefix = "☁️" if folder in temp_folders else "📂"
            lbl = QLabel(f"{prefix}  {folder}")

            btn_edit = QPushButton("✏️")
            btn_edit.setProperty("cssClass", "icon")
            btn_edit.setToolTip(tr("folder_list.edit.tooltip"))
            btn_edit.clicked.connect(lambda _, p=folder: self.on_edit(p))

            btn_del = QPushButton("✕")
            btn_del.setProperty("cssClass", "icon")
            btn_del.setToolTip(tr("folder_list.delete.tooltip"))
            btn_del.clicked.connect(lambda _, p=folder: self.on_delete(p))

            row_layout.addWidget(lbl, 1)
            row_layout.addWidget(btn_edit)
            row_layout.addWidget(btn_del)
            self.layout.addWidget(row)
