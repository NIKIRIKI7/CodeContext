import sys
import argparse
import os
import time

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.store.store import Store
from src.actions.dispatcher import Dispatcher
from src.ui.main_window import MainWindow
from src.controllers.main_controller import MainController
from src.controllers.cli_controller import CliController
from src.services.integration_service import IntegrationService
from src.utils.async_runtime import AsyncRuntime

def main():
    parser = argparse.ArgumentParser(description="CodeContext AI Entry Point")
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode")
    parser.add_argument("--path", type=str, help="Target path for CLI mode")
    parser.add_argument("--mode", type=str, choices=["default", "shallow", "deep"], default="default", help="Scan mode for files")
    parser.add_argument("--install-context", action="store_true", help="Install Context Menu")
    parser.add_argument("--remove-context", action="store_true", help="Remove Context Menu")

    args = parser.parse_args()
    integration_service = IntegrationService()

    if args.install_context:
        print("Installing context menu...")
        success, msg = integration_service.install_context_menu()
        print(msg)
        if not success:
            time.sleep(3)
        sys.exit(0)

    if args.remove_context:
        print("Removing context menu...")
        success, msg = integration_service.remove_context_menu()
        print(msg)
        if not success:
            time.sleep(3)
        sys.exit(0)

    if args.cli:
        if not args.path:
            print("Error: --path is required for CLI mode")
            sys.exit(1)
        cli = CliController()
        cli.run(args.path, args.mode)
        sys.exit(0)

    AsyncRuntime.start()
    store = Store()
    dispatcher = Dispatcher(store)
    controller = MainController(store, dispatcher)
    app = MainWindow(store, controller)
    app.protocol("WM_DELETE_WINDOW", lambda: _on_close(app))
    app.mainloop()

def _on_close(app):
    """Корректное завершение: закрытие окна и остановка цикла"""
    try:
        app.on_closing()
    except:
        pass
    AsyncRuntime.stop()
    sys.exit(0)

if __name__ == "__main__":
    main()