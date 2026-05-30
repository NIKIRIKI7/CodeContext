import argparse
import os
import sys

sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from src.di_container import DIContainer
from src.utils.async_runtime import AsyncRuntime
from src.utils.logger import app_logger
from PySide6.QtWidgets import QApplication
from PySide6.QtGui import QIcon, QImage, QPainter, QColor, QFont, QPen
from PySide6.QtCore import Qt

from src.ui.theme_manager import ThemeManager
from src.ui.main_window import MainWindow


def create_default_logo(path: str):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img = QImage(256, 256, QImage.Format_ARGB32)
    img.fill(Qt.transparent)

    painter = QPainter(img)
    painter.setRenderHint(QPainter.Antialiasing)

    painter.setBrush(QColor("#18181b"))
    painter.setPen(Qt.NoPen)
    painter.drawRoundedRect(12, 12, 232, 232, 48, 48)

    painter.setBrush(QColor("#0a84ff"))
    painter.drawRoundedRect(24, 24, 208, 208, 36, 36)

    painter.setPen(QPen(Qt.white, 16))
    font = QFont("Consolas", 64, QFont.Bold)
    painter.setFont(font)
    painter.drawText(img.rect(), Qt.AlignCenter, "{ C }")

    painter.end()
    img.save(path, "PNG")


def main():
    app_logger.info("=" * 50)
    app_logger.info("🚀 CodeContext AI Started")

    if getattr(sys, 'frozen', False):
        exe_path = sys.executable
        old_path = exe_path + ".old"
        if os.path.exists(old_path):
            try:
                os.remove(old_path)
            except Exception:
                pass

    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", action="store_true")
    parser.add_argument("--path", type=str, help="Путь к проекту")
    parser.add_argument("--patch", type=str, help="Путь к JSON-файлу с патчами от LLM")
    parser.add_argument("--mode", type=str, default="default")
    parser.add_argument("--install-context", action="store_true")
    parser.add_argument("--remove-context", action="store_true")
    args, _ = parser.parse_known_args()
    
    container = DIContainer()
    
    if args.install_context:
        container.integration_service.install_context_menu()
        sys.exit(0)

    if args.remove_context:
        container.integration_service.remove_context_menu()
        sys.exit(0)
    
    if args.cli:
        if not args.path:
            print("❌ Укажите путь к проекту через --path")
            sys.exit(1)
        if args.patch:
            container.cli_controller.run_patch(args.path, args.patch)
        else:
            container.cli_controller.run(args.path, mode=args.mode)
        sys.exit(0)

    app_logger.info("Starting GUI Mode (PySide6)...")
    AsyncRuntime.start()

    app = QApplication(sys.argv)

    base_dir = os.path.dirname(os.path.abspath(__file__))
    logo_dir = os.path.join(base_dir, "assets", "images")
    logo_path = os.path.join(logo_dir, "logo.png")

    if not os.path.exists(logo_path):
        try:
            create_default_logo(logo_path)
        except Exception as e:
            app_logger.error(f"Failed to generate logo: {e}")

    if os.path.exists(logo_path):
        app.setWindowIcon(QIcon(logo_path))

    themes_dir = os.path.join(base_dir, "themes")
    ThemeManager.load_themes(themes_dir)
    if "apple" in ThemeManager.get_available_themes():
        ThemeManager.apply_theme("apple", "light")
    elif ThemeManager.get_available_themes():
        ThemeManager.apply_theme(ThemeManager.get_available_themes()[0], "light")

    window = MainWindow(container.store, container.main_controller)
    if os.path.exists(logo_path):
        window.setWindowIcon(QIcon(logo_path))

    window.show()
    exit_code = app.exec()

    AsyncRuntime.stop()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()