"""
main.py — точка входа приложения.

Исправлено:
- Используется DIContainer как единственная точка сборки (DIP).
- Убрано ручное создание Store/Dispatcher/Controller в main().
- CLI-режим также использует DIContainer (единый граф зависимостей).
"""

import sys
import argparse
import os
import time

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.di_container import DIContainer
from src.utils.async_runtime import AsyncRuntime


def main():
    parser = argparse.ArgumentParser(description="CodeContext AI Entry Point")
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode")
    parser.add_argument("--path", type=str, help="Target path for CLI mode")
    parser.add_argument(
        "--mode",
        type=str,
        choices=["default", "shallow", "deep"],
        default="default",
        help="Scan mode for files",
    )
    parser.add_argument("--install-context", action="store_true", help="Install Context Menu")
    parser.add_argument("--remove-context", action="store_true", help="Remove Context Menu")
    args = parser.parse_args()

    # DIContainer — единственное место создания объектов
    container = DIContainer()

    # --- Context Menu (не требует UI или AsyncRuntime) ---
    if args.install_context:
        print("Installing context menu...")
        success, msg = container.integration_service.install_context_menu()
        print(msg)
        if not success:
            time.sleep(3)
        sys.exit(0)

    if args.remove_context:
        print("Removing context menu...")
        success, msg = container.integration_service.remove_context_menu()
        print(msg)
        if not success:
            time.sleep(3)
        sys.exit(0)

    # --- CLI mode ---
    if args.cli:
        if not args.path:
            print("Error: --path is required for CLI mode")
            sys.exit(1)
        container.cli_controller.run(args.path, args.mode)
        sys.exit(0)

    # --- GUI mode ---
    AsyncRuntime.start()

    # Импорт UI только когда нужен (не загружать tkinter в CLI)
    from src.ui.main_window import MainWindow

    app = MainWindow(container.store, container.main_controller)
    app.protocol("WM_DELETE_WINDOW", lambda: _on_close(app))
    app.mainloop()


def _on_close(app):
    """Корректное завершение: закрытие окна и остановка loop."""
    try:
        app.on_closing()
    except Exception:
        pass
    AsyncRuntime.stop()
    sys.exit(0)


if __name__ == "__main__":
    main()