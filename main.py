import sys
import argparse
import os

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
    parser.add_argument("--install-context", action="store_true", help="Install Context Menu")
    parser.add_argument("--remove-context", action="store_true", help="Remove Context Menu")
    args = parser.parse_args()

    # CLI и Context Menu логика остается прежней (синхронной или требует отдельного запуска)
    if args.install_context:
        # ...
        sys.exit(0)

    # Для GUI режима
    if not args.cli:
        # 1. Запускаем глобальный Async Loop в отдельном потоке
        AsyncRuntime.start()

        # 2. Инициализация слоев
        store = Store()
        dispatcher = Dispatcher(store)
        controller = MainController(store, dispatcher)

        # 3. GUI
        app = MainWindow(store, controller)
        app.protocol("WM_DELETE_WINDOW", lambda: _on_close(app))
        app.mainloop()


def _on_close(app):
    """Корректное завершение: закрытие окна и остановка цикла"""
    app.on_closing()
    AsyncRuntime.stop()
    sys.exit(0)


if __name__ == "__main__":
    main()