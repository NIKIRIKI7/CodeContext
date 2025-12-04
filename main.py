import sys
import argparse
import os

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.store.store import Store
from src.actions.dispatcher import Dispatcher
from src.ui.main_window import MainWindow
from src.controllers.main_controller import MainController  # Новый
from src.controllers.cli_controller import CliController
from src.services.integration_service import IntegrationService


def main():
    parser = argparse.ArgumentParser(description="CodeContext AI Entry Point")
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode")
    parser.add_argument("--path", type=str, help="Target path for CLI mode")
    parser.add_argument("--install-context", action="store_true", help="Install Context Menu")
    parser.add_argument("--remove-context", action="store_true", help="Remove Context Menu")
    args = parser.parse_args()

    # System Commands
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

    # CLI Mode
    if args.cli and args.path:
        controller = CliController()
        controller.run(args.path)
        sys.exit(0)

    # GUI Mode
    store = Store()
    dispatcher = Dispatcher(store)

    # Инициализация контроллера
    controller = MainController(store, dispatcher)

    # Передаем контроллер во View
    app = MainWindow(store, controller)
    app.protocol("WM_DELETE_WINDOW", app.on_closing)
    app.mainloop()


if __name__ == "__main__":
    main()