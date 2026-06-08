import argparse
import os
import sys
import platform

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.di_container import DIContainer
from src.utils.async_runtime import AsyncRuntime
from src.utils.logger import app_logger
from PySide6.QtWidgets import QApplication
from PySide6.QtGui import QIcon, QImage, QPainter, QColor, QFont, QPen
from PySide6.QtCore import Qt
from src.ui.theme_manager import ThemeManager
from src.ui.main_window import MainWindow


def get_resource_path(relative_path: str) -> str:
    """Универсальная функция для получения абсолютного пути к ресурсам (Themes, Assets)"""
    if getattr(sys, 'frozen', False):
        base_path = sys._MEIPASS
    else:
        base_path = os.path.dirname(os.path.abspath(__file__))
    return os.path.join(base_path, relative_path)


def create_default_logo(path: str):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img = QImage(256, 256, QImage.Format_ARGB32)
    img.fill(Qt.transparent)
    painter = QPainter(img)
    painter.setRenderHint(QPainter.Antialiasing)
    painter.setBrush(QColor("#0a84ff"))
    painter.setPen(Qt.NoPen)
    painter.drawRoundedRect(12, 12, 232, 232, 48, 48)
    painter.setBrush(QColor("#000000"))
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
        if platform.system() == "Darwin":
            parts = exe_path.split(os.sep)
            if "Contents" in parts and "MacOS" in parts:
                app_idx = parts.index("Contents") - 1
                app_path = os.sep.join(parts[:app_idx+1])
                old_app = app_path + ".old"
                if os.path.exists(old_app):
                    import shutil
                    try: shutil.rmtree(old_app, ignore_errors=True)
                    except Exception: pass
        else:
            old_path = exe_path + ".old"
            if os.path.exists(old_path):
                try: os.remove(old_path)
                except Exception: pass

    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", action="store_true", help="Запуск в режиме командной строки")
    parser.add_argument("--path", type=str, help="Путь к проекту")
    parser.add_argument("--patch", type=str, help="Путь к JSON-файлу с патчами от LLM")
    parser.add_argument("--mode", type=str, default="default", help="Режим сбора зависимостей (default/shallow/deep)")
    parser.add_argument("--install-context", action="store_true", help="Установить интеграцию с ОС")
    parser.add_argument("--remove-context", action="store_true", help="Удалить интеграцию с ОС")
    parser.add_argument("--minify", type=str, choices=["true", "false"], help="Включить/выключить Minify")
    parser.add_argument("--skeleton", type=str, choices=["true", "false"], help="Включить/выключить Skeleton")
    parser.add_argument("--format", type=str, choices=["markdown", "xml", "plain"], help="Формат вывода")
    parser.add_argument("--dry-run", action="store_true", help="Оценить токены без генерации (только CLI)")
    parser.add_argument("--silent", action="store_true", help="Тихий режим: без логов, скопировать сразу в буфер (только CLI)")

    args, _ = parser.parse_known_args()
    container = DIContainer()

    if args.install_context:
        container.integration_service.install_context_menu()
        sys.exit(0)

    if args.remove_context:
        container.integration_service.remove_context_menu()
        sys.exit(0)

    if args.cli or args.silent or args.dry_run:
        if not args.path:
            args.path = os.getcwd()
        kwargs = {
            'mode': args.mode,
            'minify': args.minify,
            'skeleton': args.skeleton,
            'format': args.format,
            'dry_run': args.dry_run,
            'silent': args.silent
        }
        if args.patch:
            container.cli_controller.run_patch(args.path, args.patch)
        else:
            container.cli_controller.run(args.path, **kwargs)
        sys.exit(0)

    app_logger.info("Starting GUI Mode (PySide6)...")

    os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"
    os.environ["QT_ENABLE_HIGHDPI_SCALING"] = "1"

    AsyncRuntime.start()
    app = QApplication(sys.argv)

    logo_path = get_resource_path(os.path.join("assets", "images", "logo.png"))
    if not os.path.exists(logo_path):
        try:
            create_default_logo(logo_path)
        except Exception as e:
            app_logger.error(f"Failed to generate logo: {e}")

    if os.path.exists(logo_path):
        app.setWindowIcon(QIcon(logo_path))

    built_in_themes = get_resource_path("themes")
    from src.utils.config import get_app_data_dir
    user_themes_dir = os.path.join(get_app_data_dir(), "themes")
    os.makedirs(user_themes_dir, exist_ok=True)

    ThemeManager.load_themes(built_in_themes, user_themes_dir)

    if "apple" in ThemeManager.get_available_themes():
        ThemeManager.apply_theme("apple", "light")
    elif ThemeManager.get_available_themes():
        ThemeManager.apply_theme(ThemeManager.get_available_themes()[0], "light")

    window = MainWindow(container.store, container.main_controller)
    if os.path.exists(logo_path):
        window.setWindowIcon(QIcon(logo_path))

    if args.path and os.path.exists(args.path):
        container.main_controller.add_folder(os.path.abspath(args.path))

    window.show()
    exit_code = app.exec()

    AsyncRuntime.stop()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
