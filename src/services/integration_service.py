import sys
import os
import winreg
import ctypes
from typing import Tuple


class IntegrationService:
    """Сервис для интеграции с Windows (Реестр, Права администратора)"""

    def is_admin(self) -> bool:
        """Проверяет, запущен ли процесс с правами администратора"""
        try:
            return ctypes.windll.shell32.IsUserAnAdmin()
        except:
            return False

    def restart_as_admin(self, extra_arg: str):
        """
        Перезапускает текущий скрипт с запросом прав администратора (UAC)
        и добавляет аргумент действия (например, --install-context).
        """
        if self.is_admin():
            return

        # Определяем, как мы запущены
        if getattr(sys, 'frozen', False):
            # Если это скомпилированный EXE
            executable = sys.executable
            params = extra_arg
        else:
            # Если это Python скрипт
            executable = sys.executable
            # Получаем абсолютный путь к main.py
            # Важно: sys.argv[0] может быть относительным, делаем его абсолютным
            script_path = os.path.abspath(sys.argv[0])
            # Формируем команду: "путь/к/main.py" --флаг
            params = f'"{script_path}" {extra_arg}'

        try:
            # Запуск через ShellExecute с глаголом "runas" (запрос прав)
            # Параметр 1 (SW_SHOWNORMAL) показывает окно консоли, чтобы пользователь видел результат
            ctypes.windll.shell32.ShellExecuteW(None, "runas", executable, params, None, 1)
        except Exception as e:
            print(f"Error elevating privileges: {e}")

    def install_context_menu(self) -> Tuple[bool, str]:
        """Регистрация пункта в контекстном меню"""

        # 1. Если нет прав, запускаем отдельный процесс-администратор с флагом установки
        if not self.is_admin():
            self.restart_as_admin("--install-context")
            return False, "Запрошены права администратора. Проверьте открывшееся окно."

        # 2. Если права есть, выполняем запись в реестр
        try:
            if getattr(sys, 'frozen', False):
                exe_path = sys.executable
                command = f'"{exe_path}" --cli --path "%1"'
                icon_path = exe_path
            else:
                python_exe = sys.executable
                script_path = os.path.abspath(sys.argv[0])

                # Костыль: если запускаем из src или другой папки, ищем main.py правильно
                if not script_path.endswith("main.py"):
                    # Пытаемся найти main.py в текущей директории
                    possible = os.path.join(os.getcwd(), "main.py")
                    if os.path.exists(possible):
                        script_path = possible

                # Используем pythonw.exe для команды в реестре, чтобы при клике ПКМ не мигала консоль,
                # либо python.exe если хотим видеть процесс.
                # Для CLI режима обычно лучше python.exe, чтобы видеть ошибки,
                # или реализовать логирование в файл.
                command = f'"{python_exe}" "{script_path}" --cli --path "%1"'
                icon_path = python_exe

            key_path = r"Directory\shell\CodeContextAI"

            key = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, key_path)
            winreg.SetValue(key, "", winreg.REG_SZ, "Scan with CodeContext AI")
            winreg.SetValueEx(key, "Icon", 0, winreg.REG_SZ, icon_path)

            command_key = winreg.CreateKey(key, "command")
            winreg.SetValue(command_key, "", winreg.REG_SZ, command)

            winreg.CloseKey(command_key)
            winreg.CloseKey(key)

            return True, "Успешно! Пункт меню добавлен."

        except Exception as e:
            return False, f"Ошибка записи в реестр: {e}"

    def remove_context_menu(self) -> Tuple[bool, str]:
        """Удаление пункта из контекстного меню"""

        # 1. Если нет прав, перезапускаем с флагом удаления
        if not self.is_admin():
            self.restart_as_admin("--remove-context")
            return False, "Запрошены права администратора. Проверьте открывшееся окно."

        # 2. Удаляем ключи
        key_path = r"Directory\shell\CodeContextAI"
        try:
            try:
                winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, key_path + r"\command")
            except FileNotFoundError:
                pass

            winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, key_path)
            return True, "Успешно! Пункт меню удален."

        except FileNotFoundError:
            return True, "Пункта меню уже нет."
        except Exception as e:
            return False, f"Ошибка удаления из реестра: {e}"