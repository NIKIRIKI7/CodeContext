import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QTableWidget, QTableWidgetItem, \
    QHeaderView, QProgressBar, QHBoxLayout
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager, theme_bus


class AnalyticsPanel(QWidget):
    def __init__(self):
        super().__init__()
        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)

        self.table = QTableWidget(0, 3)
        self.table.setHorizontalHeaderLabels(["Файл", "Токены", "Вес (Прогресс)"])

        header = self.table.horizontalHeader()
        header.setStretchLastSection(False)
        header.setSectionResizeMode(0, QHeaderView.Stretch)
        header.setSectionResizeMode(1, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(2, QHeaderView.Interactive)

        self.table.setEditTriggers(QTableWidget.NoEditTriggers)
        self.table.setSelectionBehavior(QTableWidget.SelectRows)
        self.table.setShowGrid(False)
        self.table.verticalHeader().setVisible(False)

        self.layout.addWidget(self.table)

        self._apply_theme()
        theme_bus.theme_changed.connect(self._apply_theme)

    def _get_theme_size_int(self, category: str, key: str, default: int) -> int:
        theme = ThemeManager._themes.get(ThemeManager._current_theme, {})
        val_str = theme.get("default_styles", {}).get(category, {}).get(key, f"{default}px")
        try:
            return int(str(val_str).replace("px", "").strip())
        except ValueError:
            return default

    def _apply_theme(self):
        colors = ThemeManager.get_current_colors()

        c_bg = colors.get('card', '#ffffff')
        c_text = colors.get('text', '#000000')
        c_border = colors.get('border', '#cccccc')
        c_hover = colors.get('secondary', '#eeeeee')

        f_size = self._get_theme_size_int("fonts", "size", 14)
        row_height = int(f_size * 2.5)
        self.table.verticalHeader().setDefaultSectionSize(row_height)
        self.table.setColumnWidth(2, int(f_size * 10))

        self.table.setStyleSheet(f"""
            QTableWidget {{ 
                border: none; 
                background: {c_bg}; 
                color: {c_text};
            }}
            QHeaderView::section {{
                font-weight: bold;
                border: none;
                border-bottom: 2px solid {c_border};
                padding: 4px;
                background: {c_bg};
                color: {c_text};
            }}
            QTableWidget::item {{
                border-bottom: 1px solid {c_border};
                padding-right: 8px;
            }}
            QTableWidget::item:selected {{
                background-color: {c_hover};
                color: {c_text};
            }}
        """)

        self._update_progress_bars_style()

    def _update_progress_bars_style(self):
        colors = ThemeManager.get_current_colors()
        c_primary = colors.get('primary', '#0071e3')
        c_success = colors.get('success', '#34c759')
        c_danger = colors.get('danger', '#ff3b30')
        c_track = colors.get('secondary', '#e8e8ed')

        prog_radius = self._get_theme_size_int("radii", "progress", 4)

        for row in range(self.table.rowCount()):
            bar_container = self.table.cellWidget(row, 2)
            if not bar_container:
                continue

            bar = bar_container.findChild(QProgressBar)
            if not bar:
                continue

            tokens_item = self.table.item(row, 1)
            if not tokens_item:
                continue

            try:
                tokens = int(tokens_item.text().replace(" tk", "").strip())
            except ValueError:
                tokens = 0

            color = c_primary
            if tokens > 50000:
                color = c_danger
            elif tokens > 25000:
                color = c_success

            bar.setStyleSheet(f"""
                QProgressBar {{ 
                    background-color: {c_track}; 
                    border: none; 
                    border-radius: {prog_radius}px; 
                }}
                QProgressBar::chunk {{ 
                    background-color: {color}; 
                    border-radius: {prog_radius}px; 
                }}
            """)

    def populate(self, metadata: dict, manual_exclusions: set):
        self.table.setRowCount(0)

        valid_files = [
            (path, meta['tokens']) for path, meta in metadata.items()
            if path not in manual_exclusions
        ]
        valid_files.sort(key=lambda x: x[1], reverse=True)

        if not valid_files:
            return

        max_tokens = valid_files[0][1] if valid_files[0][1] > 0 else 1

        colors = ThemeManager.get_current_colors()
        c_primary = colors.get('primary', '#0071e3')
        c_success = colors.get('success', '#34c759')
        c_danger = colors.get('danger', '#ff3b30')
        c_track = colors.get('secondary', '#e8e8ed')

        prog_height = self._get_theme_size_int("sizes", "progress_height", 6)
        prog_radius = self._get_theme_size_int("radii", "progress", 4)

        for row, (path, tokens) in enumerate(valid_files[:100]):
            self.table.insertRow(row)

            filename = os.path.basename(path)
            item_name = QTableWidgetItem(f"📄 {filename}")
            item_name.setToolTip(path)
            item_name.setTextAlignment(Qt.AlignLeft | Qt.AlignVCenter)
            self.table.setItem(row, 0, item_name)

            item_tokens = QTableWidgetItem(f"{tokens} tk ")
            item_tokens.setTextAlignment(Qt.AlignRight | Qt.AlignVCenter)
            self.table.setItem(row, 1, item_tokens)

            bar_container = QWidget()
            bar_layout = QVBoxLayout(bar_container)
            bar_layout.setContentsMargins(10, 0, 10, 0)
            bar_layout.setAlignment(Qt.AlignCenter)

            bar = QProgressBar()
            bar.setRange(0, max_tokens)
            bar.setValue(tokens)
            bar.setTextVisible(False)
            bar.setFixedHeight(prog_height)

            color = c_primary
            if tokens > 50000:
                color = c_danger
            elif tokens > 25000:
                color = c_success

            bar.setStyleSheet(f"""
                QProgressBar {{ 
                    background-color: {c_track}; 
                    border: none; 
                    border-radius: {prog_radius}px; 
                }}
                QProgressBar::chunk {{ 
                    background-color: {color}; 
                    border-radius: {prog_radius}px; 
                }}
            """)

            bar_layout.addWidget(bar)
            self.table.setCellWidget(row, 2, bar_container)
