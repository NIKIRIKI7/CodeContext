import os
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QPushButton,
                               QFrame, QGridLayout)
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus


class RecentCard(QFrame):
    """Карточка 'Недавнего проекта'. Ведет себя как кликабельная кнопка."""

    def __init__(self, path, on_click):
        super().__init__()
        self.path = path
        self.on_click = on_click
        self.setProperty("cssClass", "recent_card")
        self.setCursor(Qt.PointingHandCursor)

        layout = QVBoxLayout(self)
        layout.setSpacing(4)

        name_lbl = QLabel(os.path.basename(path) or path)
        name_lbl.setProperty("cssClass", "recent_name")

        path_lbl = QLabel(path)
        path_lbl.setProperty("cssClass", "recent_path")

        layout.addWidget(name_lbl)
        layout.addWidget(path_lbl)

    def mousePressEvent(self, event):
        if event.button() == Qt.LeftButton:
            self.on_click(self.path)
        super().mousePressEvent(event)


class EmptyState(QWidget):
    def __init__(self, on_folder_select):
        super().__init__()
        self.on_folder_select = on_folder_select
        self.layout = QVBoxLayout(self)

        # 1. Большая зона Drag & Drop
        self.drop_zone = QFrame()
        self.drop_zone.setProperty("cssClass", "drop_zone")
        drop_layout = QVBoxLayout(self.drop_zone)
        drop_layout.setAlignment(Qt.AlignCenter)

        icon_lbl = QLabel("📁")
        icon_lbl.setProperty("cssClass", "empty_icon")
        icon_lbl.setAlignment(Qt.AlignCenter)

        title_lbl = QLabel("Перетащите папку проекта сюда")
        title_lbl.setProperty("cssClass", "empty_title")
        title_lbl.setAlignment(Qt.AlignCenter)

        subtitle_lbl = QLabel("или выберите вручную, нажав кнопку ниже")
        subtitle_lbl.setProperty("cssClass", "empty_subtitle")
        subtitle_lbl.setAlignment(Qt.AlignCenter)

        btn_browse = QPushButton("Обзор файлов")
        btn_browse.setProperty("cssClass", "success")
        btn_browse.setCursor(Qt.PointingHandCursor)
        btn_browse.clicked.connect(self._browse)

        drop_layout.addStretch()
        drop_layout.addWidget(icon_lbl)
        drop_layout.addSpacing(16)
        drop_layout.addWidget(title_lbl)
        drop_layout.addSpacing(8)
        drop_layout.addWidget(subtitle_lbl)
        drop_layout.addSpacing(24)
        drop_layout.addWidget(btn_browse, 0, Qt.AlignCenter)
        drop_layout.addStretch()

        # 2. Недавние проекты
        self.recent_container = QWidget()
        self.recent_layout = QVBoxLayout(self.recent_container)
        self.recent_layout.setContentsMargins(0, 0, 0, 0)

        recent_title = QLabel("Последние проекты")
        recent_title.setProperty("cssClass", "heading")
        self.recent_layout.addWidget(recent_title)

        self.recent_grid = QGridLayout()
        self.recent_grid.setSpacing(12)
        self.recent_layout.addLayout(self.recent_grid)

        # Компоновка
        self.layout.addStretch(1)
        self.layout.addWidget(self.drop_zone, 4)
        self.layout.addSpacing(32)
        self.layout.addWidget(self.recent_container, 2)
        self.layout.addStretch(2)

        self._update_metrics()
        theme_bus.theme_changed.connect(self._update_metrics)

    def _update_metrics(self):
        m = ThemeManager.get_layout("panel_margin", 20)
        self.layout.setContentsMargins(m, m, m, m)

    def update_recent(self, recent_paths):
        # Очистка предыдущих карточек
        for i in reversed(range(self.recent_grid.count())):
            item = self.recent_grid.itemAt(i)
            if item.widget():
                item.widget().deleteLater()

        if not recent_paths:
            self.recent_container.hide()
            return

        self.recent_container.show()
        row, col = 0, 0
        max_cols = 2
        added = 0

        for path in recent_paths:
            if not os.path.exists(path):
                continue
            card = RecentCard(path, self.on_folder_select)
            self.recent_grid.addWidget(card, row, col)
            added += 1
            col += 1
            if col >= max_cols:
                col = 0
                row += 1

        if added == 0:
            self.recent_container.hide()

    def _browse(self):
        from PySide6.QtWidgets import QFileDialog
        path = QFileDialog.getExistingDirectory(self, "Выберите папку проекта")
        if path:
            self.on_folder_select(path)