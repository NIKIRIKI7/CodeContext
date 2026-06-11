import os
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QPushButton,
                               QFrame, QGridLayout)
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus
from src.i18n import tr


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
        
        self.title_lbl = QLabel(tr("empty_state.drop.title"))
        self.title_lbl.setProperty("cssClass", "empty_title")
        self.title_lbl.setAlignment(Qt.AlignCenter)
        
        self.subtitle_lbl = QLabel(tr("empty_state.drop.subtitle"))
        self.subtitle_lbl.setProperty("cssClass", "empty_subtitle")
        self.subtitle_lbl.setAlignment(Qt.AlignCenter)
        
        self.btn_browse = QPushButton(tr("empty_state.browse.button"))
        self.btn_browse.setProperty("cssClass", "success")
        self.btn_browse.setCursor(Qt.PointingHandCursor)
        self.btn_browse.clicked.connect(self._browse)
        
        drop_layout.addStretch()
        drop_layout.addWidget(icon_lbl)
        drop_layout.addSpacing(16)
        drop_layout.addWidget(self.title_lbl)
        drop_layout.addSpacing(8)
        drop_layout.addWidget(self.subtitle_lbl)
        drop_layout.addSpacing(24)
        drop_layout.addWidget(self.btn_browse, 0, Qt.AlignCenter)
        drop_layout.addStretch()

        self.recent_container = QWidget()
        self.recent_layout = QVBoxLayout(self.recent_container)
        self.recent_layout.setContentsMargins(0, 0, 0, 0)
        
        self.recent_title = QLabel(tr("empty_state.recent.title"))
        self.recent_title.setProperty("cssClass", "heading")
        self.recent_layout.addWidget(self.recent_title)

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

    def retranslate_ui(self):
        self.title_lbl.setText(tr("empty_state.drop.title"))
        self.subtitle_lbl.setText(tr("empty_state.drop.subtitle"))
        self.btn_browse.setText(tr("empty_state.browse.button"))
        self.recent_title.setText(tr("empty_state.recent.title"))

    def _browse(self):
        from PySide6.QtWidgets import QFileDialog
        path = QFileDialog.getExistingDirectory(self, tr("empty_state.browse.dialog_title"))
        if path:
            self.on_folder_select(path)