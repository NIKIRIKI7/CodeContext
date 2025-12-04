import sys
import argparse
import os

# Добавляем путь к src, чтобы импорты работали корректно из любого места
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.store.store import Store
from src.actions.dispatcher import Dispatcher
from src.ui.main_window import MainWindow
from src.services.integration_service import IntegrationService
from src.controllers.cli_controller import CliController


def main():
    # 1. Настройка аргументов командной строки
    parser = argparse.ArgumentParser(description="CodeContext AI Entry Point")

    # Режимы работы
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode")
    parser.add_argument("--path", type=str, help="Target path for CLI mode")

    # Системные команды (вызываются из UI)
    parser.add_argument("--install-context", action="store_true", help="Install Context Menu")
    parser.add_argument("--remove-context", action="store_true", help="Remove Context Menu")

    args = parser.parse_args()

    # 2. Обработка системных команд (Установка/Удаление меню)
    # Эти команды выполняются быстро и закрывают процесс
    if args.install_context:
        print(">> Установка контекстного меню...")
        service = IntegrationService()
        success, msg = service.install_context_menu()
        print(f"\n{msg}")
        input("\nНажмите Enter, чтобы закрыть...")
        sys.exit(0)

    if args.remove_context:
        print(">> Удаление контекстного меню...")
        service = IntegrationService()
        success, msg = service.remove_context_menu()
        print(f"\n{msg}")
        input("\nНажмите Enter, чтобы закрыть...")
        sys.exit(0)

    # 3. Запуск режима CLI (Headless)
    if args.cli and args.path:
        controller = CliController()
        controller.run(args.path)
        sys.exit(0)

    # 4. Запуск режима GUI (Обычный)
    # Инициализация Store и Dispatcher происходит здесь, так как они нужны только UI
    store = Store()
    dispatcher = Dispatcher(store)

    app = MainWindow(store, dispatcher)
    # Корректная обработка закрытия окна
    app.protocol("WM_DELETE_WINDOW", app.on_closing)
    app.mainloop()


if __name__ == "__main__":
    main()