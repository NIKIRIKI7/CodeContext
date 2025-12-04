import sys
import argparse
import ctypes
import os

# Добавляем текущую директорию в путь, чтобы импорты работали корректно
# даже при запуске от администратора из другой папки
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.store.store import Store
from src.actions.dispatcher import Dispatcher
from src.ui.main_window import MainWindow
from src.services.integration_service import IntegrationService


def main():
    # 1. Парсинг аргументов
    parser = argparse.ArgumentParser(description="CodeContext AI")
    parser.add_argument("--cli", action="store_true", help="Run in headless CLI mode for scanning")
    parser.add_argument("--path", type=str, help="Path to process (for CLI mode)")

    # Новые флаги для админских задач
    parser.add_argument("--install-context", action="store_true", help="Install Windows Context Menu (Internal)")
    parser.add_argument("--remove-context", action="store_true", help="Remove Windows Context Menu (Internal)")

    args = parser.parse_args()

    # 2. Обработка системных команд (Интеграция)
    if args.install_context:
        print("Запуск установки контекстного меню...")
        service = IntegrationService()
        success, msg = service.install_context_menu()
        print(f"\n{msg}")
        input("\nНажмите Enter, чтобы закрыть...")
        sys.exit(0)

    if args.remove_context:
        print("Запуск удаления контекстного меню...")
        service = IntegrationService()
        success, msg = service.remove_context_menu()
        print(f"\n{msg}")
        input("\nНажмите Enter, чтобы закрыть...")
        sys.exit(0)

    # 3. Инициализация ядра приложения (для обычного режима)
    store = Store()
    dispatcher = Dispatcher(store)

    # 4. Режим CLI (Сканирование через контекстное меню)
    if args.cli and args.path:
        # Здесь должна быть реализация CLI контроллера.
        # В данной архитектуре мы используем упрощенный вывод.
        # Для полноценной работы нужно создать HeadlessController.
        print("Запуск в режиме CLI...")
        # (Оставляем заглушку или можно подключить логику ProcessingService здесь)
        # В рамках текущей задачи это не меняем.
        pass

    # 5. Режим GUI (Обычный запуск)
    else:
        app = MainWindow(store, dispatcher)
        app.protocol("WM_DELETE_WINDOW", app.on_closing)
        app.mainloop()


if __name__ == "__main__":
    main()