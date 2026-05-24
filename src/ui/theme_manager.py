import json
import os
import base64
from PySide6.QtWidgets import QApplication
from PySide6.QtCore import QObject, Signal


class ThemeSignalBus(QObject):
    theme_changed = Signal()


theme_bus = ThemeSignalBus()


def _create_svg_url(svg_str):
    """Кодирует SVG в чистый base64 без кавычек (самый надежный способ для QSS)"""
    b64 = base64.b64encode(svg_str.encode('utf-8')).decode('ascii')
    return f"url(data:image/svg+xml;base64,{b64})"


QSS_TEMPLATE = """
QWidget {{
    font-family: {f_family};
    font-size: {f_size};
    color: {c_text};
}}

QMainWindow, QDialog {{
    background-color: {c_bg};
}}

QWidget[cssClass="card"] {{
    background-color: {c_card};
    border-radius: {r_card};
    border: 1px solid {c_border};
}}

QLabel[cssClass="heading"] {{
    font-size: {f_heading};
    font-weight: bold;
    color: {c_text};
}}

QLabel[cssClass="muted"] {{
    color: {c_text_muted};
}}

QPushButton {{
    background-color: {c_primary};
    color: {c_primary_fg};
    border-radius: {r_button};
    padding: {s_pad_btn};
    font-weight: bold;
    border: none;
}}
QPushButton:hover {{
    background-color: {c_primary};
    opacity: 0.9;
}}
QPushButton:disabled {{
    background-color: {c_border};
    color: {c_text_muted};
}}
QPushButton[cssClass="ghost"] {{
    background-color: {c_secondary};
    color: {c_text};
}}
QPushButton[cssClass="ghost"]:hover {{
    background-color: {c_border};
}}
QPushButton[cssClass="success"] {{
    background-color: {c_success};
    color: white;
}}

QPushButton[cssClass="icon"] {{
    background-color: transparent;
    color: {c_text_muted};
    padding: 0px;
    border-radius: {r_small};
    font-weight: normal;
    font-size: 16px;
}}
QPushButton[cssClass="icon"]:hover {{
    background-color: {c_border};
    color: {c_text};
}}

QLineEdit, QComboBox {{
    background-color: {c_input_bg};
    color: {c_text};
    border: 1px solid {c_border};
    border-radius: {r_input};
    padding: {s_pad_input};
    min-height: 28px;
    selection-background-color: {c_primary};
    selection-color: {c_primary_fg};
}}
QLineEdit:focus, QComboBox:focus {{
    border: 1px solid {c_primary};
}}

QComboBox::drop-down {{
    subcontrol-origin: padding;
    subcontrol-position: top right;
    width: 24px;
    border-left: none;
}}
QComboBox::down-arrow {{
    image: {svg_down_arrow};
    width: 14px;
    height: 14px;
}}

QComboBox QAbstractItemView {{
    background-color: {c_card};
    color: {c_text};
    border: 1px solid {c_border};
    border-radius: {r_small};
    selection-background-color: {c_secondary};
    selection-color: {c_text};
    outline: none;
    padding: 4px;
}}
QComboBox QAbstractItemView::item {{
    min-height: 28px;
    border-radius: {r_small};
}}

QTreeView, QListWidget, QPlainTextEdit, QTextEdit, QScrollArea {{
    background-color: {c_card};
    color: {c_text};
    border: 1px solid {c_border};
    border-radius: {r_card};
    outline: none;
    selection-background-color: {c_primary};
    selection-color: {c_primary_fg};
}}

QScrollArea::viewport, QTreeView::viewport, QListWidget::viewport, QPlainTextEdit::viewport, QTextEdit::viewport {{
    background-color: transparent;
    border-radius: {r_card};
}}

QCheckBox {{
    spacing: {s_chk_space};
    color: {c_text};
}}
QCheckBox::indicator {{
    width: {sz_chk};
    height: {sz_chk};
    border-radius: {r_chk};
    border: 1px solid {c_border};
    background-color: {c_input_bg};
}}
QCheckBox::indicator:hover {{
    border: 1px solid {c_primary};
}}
QCheckBox::indicator:checked {{
    background-color: {c_primary};
    border: 1px solid {c_primary};
    image: {svg_check};
}}

QTabWidget::pane {{
    border: none;
    background-color: transparent;
}}
QTabBar::tab {{
    background: {c_bg};
    color: {c_text_muted};
    padding: {s_pad_btn};
    border-radius: {r_button};
    margin-right: {s_tab_margin};
    margin-bottom: {s_tab_margin};
    font-weight: bold;
}}
QTabBar::tab:selected {{
    background: {c_secondary};
    color: {c_text};
}}

QTreeView::item, QListWidget::item {{
    padding: 4px;
    border-radius: {r_small};
    min-height: 24px;
}}
QTreeView::item:selected, QListWidget::item:selected {{
    background-color: {c_secondary};
    color: {c_text};
}}
QTreeView::indicator {{
    width: {sz_tree_ind};
    height: {sz_tree_ind};
    border-radius: {r_chk};
    border: 1px solid {c_border};
    background-color: {c_input_bg};
}}
QTreeView::indicator:checked {{
    background-color: {c_primary};
    border: 1px solid {c_primary};
    image: {svg_check};
}}

QSplitter::handle {{
    background-color: transparent;
}}

QScrollBar:vertical {{
    border: none;
    background: transparent;
    width: {sz_scroll};
    border-radius: {r_scroll};
}}
QScrollBar::handle:vertical {{
    background: {c_border};
    min-height: 20px;
    border-radius: {r_scroll};
}}
QScrollBar::handle:vertical:hover {{
    background: {c_text_muted};
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{ height: 0px; }}

QScrollBar:horizontal {{
    border: none;
    background: transparent;
    height: {sz_scroll};
    border-radius: {r_scroll};
}}
QScrollBar::handle:horizontal {{
    background: {c_border};
    min-width: 20px;
    border-radius: {r_scroll};
}}
QScrollBar::handle:horizontal:hover {{
    background: {c_text_muted};
}}
QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal {{ width: 0px; }}

QProgressBar {{
    border: none;
    background-color: {c_border};
    border-radius: {r_prog};
    text-align: center;
    color: transparent;
    max-height: {sz_prog};
}}
QProgressBar::chunk {{
    background-color: {c_primary};
    border-radius: {r_prog};
}}
"""


class ThemeManager:
    _themes = {}
    _current_theme = None
    _current_mode = "light"

    @classmethod
    def load_themes(cls, themes_dir: str):
        if not os.path.exists(themes_dir):
            return

        for filename in os.listdir(themes_dir):
            if filename.endswith(".json"):
                path = os.path.join(themes_dir, filename)
                try:
                    with open(path, 'r', encoding='utf-8') as f:
                        data = json.load(f)
                        theme_id = filename.replace(".json", "")
                        cls._themes[theme_id] = data
                except Exception as e:
                    print(f"Error loading theme {filename}: {e}")

        if cls._themes and cls._current_theme not in cls._themes:
            cls._current_theme = list(cls._themes.keys())[0]

    @classmethod
    def get_available_themes(cls) -> list:
        return list(cls._themes.keys())

    @classmethod
    def get_layout(cls, key: str, default: int = 0) -> int:
        theme = cls._themes.get(cls._current_theme, {})
        layout = theme.get("default_styles", {}).get("layout", {})
        return int(layout.get(key, default))

    @classmethod
    def apply_theme(cls, theme_id: str = None, mode: str = None):
        app = QApplication.instance()
        if not app or not cls._themes:
            return

        if theme_id and theme_id in cls._themes:
            cls._current_theme = theme_id
        if mode in ["light", "dark"]:
            cls._current_mode = mode

        theme = cls._themes.get(cls._current_theme)
        if not theme:
            return

        modes = theme.get("modes", {})
        selected_mode_colors = modes.get(cls._current_mode, modes.get("light", {}))

        defaults = theme.get("default_styles", {})
        radii = defaults.get("radii", {})
        fonts = defaults.get("fonts", {})
        spacing = defaults.get("spacing", {})
        sizes = defaults.get("sizes", {})

        primary_fg_hex = selected_mode_colors.get("primary_fg", "#ffffff")
        text_hex = selected_mode_colors.get("text", "#000000")

        # Добавлены 'Segoe UI Emoji', 'Apple Color Emoji' для гарантии отрисовки 📂 ✏️
        base_font_family = fonts.get("family", "sans-serif")
        f_family_with_emoji = f"{base_font_family}, 'Segoe UI Emoji', 'Apple Color Emoji'"

        svg_check_str = f"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='{primary_fg_hex}' stroke-width='4' stroke-linecap='round' stroke-linejoin='round'><polyline points='20 6 9 17 4 12'/></svg>"
        svg_arrow_str = f"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='{text_hex}' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><polyline points='6 9 12 15 18 9'/></svg>"

        qss = QSS_TEMPLATE.format(
            c_bg=selected_mode_colors.get("bg", "#ffffff"),
            c_card=selected_mode_colors.get("card", "#f9f9f9"),
            c_text=text_hex,
            c_text_muted=selected_mode_colors.get("text_muted", "#888888"),
            c_primary=selected_mode_colors.get("primary", "#007bff"),
            c_primary_fg=primary_fg_hex,
            svg_check=_create_svg_url(svg_check_str),
            svg_down_arrow=_create_svg_url(svg_arrow_str),
            c_secondary=selected_mode_colors.get("secondary", "#e0e0e0"),
            c_secondary_fg=selected_mode_colors.get("secondary_fg", "#000000"),
            c_border=selected_mode_colors.get("border", "#cccccc"),
            c_success=selected_mode_colors.get("success", "#28a745"),
            c_danger=selected_mode_colors.get("danger", "#dc3545"),
            c_input_bg=selected_mode_colors.get("input_bg", "#ffffff"),

            r_card=radii.get("card", "8px"),
            r_button=radii.get("button", "6px"),
            r_input=radii.get("input", "6px"),
            r_small=radii.get("small", "4px"),
            r_chk=radii.get("checkbox", "4px"),
            r_scroll=radii.get("scrollbar", "4px"),
            r_prog=radii.get("progress", "2px"),

            f_family=f_family_with_emoji,
            f_size=fonts.get("size", "14px"),
            f_heading=fonts.get("heading_size", "18px"),

            s_pad_btn=spacing.get("padding_btn", "8px 16px"),
            s_pad_input=spacing.get("padding_input", "8px 12px"),
            s_pad_item=spacing.get("padding_item", "6px 8px"),
            s_tab_margin=spacing.get("margin_tab", "4px"),
            s_chk_space=spacing.get("checkbox_spacing", "8px"),

            sz_chk=sizes.get("checkbox_size", "18px"),
            sz_scroll=sizes.get("scrollbar_width", "8px"),
            sz_prog=sizes.get("progress_height", "4px"),
            sz_tree_ind=sizes.get("tree_indicator", "16px")
        )

        app.setStyleSheet(qss)
        theme_bus.theme_changed.emit()