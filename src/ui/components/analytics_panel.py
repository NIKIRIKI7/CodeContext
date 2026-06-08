import os
from PySide6.QtWidgets import QWidget, QVBoxLayout, QTableWidget, QTableWidgetItem, QHeaderView, QProgressBar, QHBoxLayout
from PySide6.QtCore import Qt
from ..theme_manager import ThemeManager


class AnalyticsPanel(QWidget):
    def __init__(self):
        super().__init__()
        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)

        self.table = QTableWidget(0, 3)
        self.table.setHorizontalHeaderLabels(["Файл", "Токены", "Вес"])

        header = self.table.horizontalHeader()
        header.setStretchLastSection(False)
        header.setSectionResizeMode(0, QHeaderView.Stretch)
        header.setSectionResizeMode(1, QHeaderView.Fixed)
        header.setSectionResizeMode(2, QHeaderView.Fixed)

        self.table.setColumnWidth(1, 100)
        self.table.setColumnWidth(2, 120)

        self.table.setEditTriggers(QTableWidget.NoEditTriggers)
        self.table.setSelectionBehavior(QTableWidget.SelectRows)
        self.table.setShowGrid(False)
        self.table.verticalHeader().setVisible(False)
        self.table.setAlternatingRowColors(True)

        self.table.setStyleSheet("""
            QTableWidget { border: none; background: transparent; }
            QHeaderView::section {
                font-weight: bold;
                border: none;
                border-bottom: 2px solid rgba(150, 150, 150, 0.3);
                padding: 4px;
                background: transparent;
            }
            QTableWidget::item {
                border-bottom: 1px solid rgba(150, 150, 150, 0.1);
            }
        """)

        self.layout.addWidget(self.table)

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

        for row, (path, tokens) in enumerate(valid_files[:100]):
            self.table.insertRow(row)
            self.table.setRowHeight(row, 36)

            filename = os.path.basename(path)
            item_name = QTableWidgetItem(f"📄 {filename}")
            item_name.setToolTip(path)
            item_name.setTextAlignment(Qt.AlignLeft | Qt.AlignVCenter)
            self.table.setItem(row, 0, item_name)

            item_tokens = QTableWidgetItem(f"{tokens} tk ")
            item_tokens.setTextAlignment(Qt.AlignRight | Qt.AlignVCenter)
            self.table.setItem(row, 1, item_tokens)

            bar_container = QWidget()
            bar_layout = QHBoxLayout(bar_container)
            bar_layout.setContentsMargins(10, 0, 10, 0)
            bar_layout.setAlignment(Qt.AlignCenter)

            bar = QProgressBar()
            bar.setRange(0, max_tokens)
            bar.setValue(tokens)
            bar.setTextVisible(False)
            bar.setFixedHeight(8)

            colors = ThemeManager.get_current_colors()
            color = colors.get('primary', '#0071e3')
            if tokens > 50000:
                color = colors.get('danger', '#ff3b30')
            elif tokens > 25000:
                color = colors.get('success', '#34c759')

            bar.setStyleSheet(f"""
                QProgressBar {{ background-color: rgba(150, 150, 150, 0.2); border: none; border-radius: 4px; }}
                QProgressBar::chunk {{ background-color: {color}; border-radius: 4px; }}
            """)

            bar_layout.addWidget(bar)
            self.table.setCellWidget(row, 2, bar_container)
