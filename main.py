import argparse
import os
import sys

sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from src.di_container import DIContainer
from src.utils.async_runtime import AsyncRuntime
from src.utils.logger import app_logger
from PySide6.QtWidgets import QApplication

from src.ui.theme_manager import ThemeManager
from src.ui.main_window import MainWindow


def main():
    app_logger.info("=" * 50)
    app_logger.info("🚀 CodeContext AI Started")

    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", action="store_true")
    parser.add_argument("--path", type=str, help="Путь к проекту")
    parser.add_argument("--patch", type=str, help="Путь к JSON-файлу с патчами от LLM")
    parser.add_argument("--mode", type=str, default="default")
    args, _ = parser.parse_known_args()

    container = DIContainer()

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

    # 1. Запуск asyncio loop в фоне
    AsyncRuntime.start()

    # 2. Инициализация Qt
    app = QApplication(sys.argv)

    # 3. Загрузка JSON тем
    base_dir = os.path.dirname(os.path.abspath(__file__))
    themes_dir = os.path.join(base_dir, "themes")
    ThemeManager.load_themes(themes_dir)

    # 4. Применение темы по умолчанию
    # Пробуем загрузить apple, если ее нет - любую доступную
    if "apple" in ThemeManager.get_available_themes():
        ThemeManager.apply_theme("apple", "light")
    elif ThemeManager.get_available_themes():
        ThemeManager.apply_theme(ThemeManager.get_available_themes()[0], "light")

    # 5. Создание и запуск главного окна
    window = MainWindow(container.store, container.main_controller)
    window.show()

    exit_code = app.exec()

    # 6. Очистка ресурсов
    AsyncRuntime.stop()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()