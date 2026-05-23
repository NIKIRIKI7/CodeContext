import sys
import os
import winreg
import ctypes
from typing import Tuple


class IntegrationService:
    """Сервис для интеграции с Windows (Реестр, Права администратора)"""

    @staticmethod
    def is_admin() -> bool:
        """Проверяет, запущен ли процесс с правами администратора"""
        try:
            return ctypes.windll.shell32.IsUserAnAdmin()
        except:
            return False

    @staticmethod
    def restart_as_admin(extra_arg: str):
        """
        Перезапускает текущий скрипт с запросом прав администратора (UAC)
        и добавляет аргумент действия (например, --install-context).
        """
        # Если уже админ, не перезапускаем, чтобы избежать вечного цикла,
        # если проблема не в правах
        if getattr(sys, 'frozen', False):
            executable = sys.executable
            params = extra_arg
        else:
            executable = sys.executable
            script_path = os.path.abspath(sys.argv[0])
            params = f'"{script_path}" {extra_arg}'

        try:
            ctypes.windll.shell32.ShellExecuteW(None, "runas", executable, params, None, 1)
        except Exception as e:
            print(f"Error elevating privileges: {e}")

    def _delete_registry_tree(self, root_key, key_path):
        """
        Рекурсивно удаляет ключ реестра и все его подклавиши.
        Необходим, так как winreg.DeleteKey падает с [WinError 5],
        если внутри ключа есть что-то еще.
        """
        try:
            open_key = winreg.OpenKey(root_key, key_path, 0, winreg.KEY_ALL_ACCESS)
            info = winreg.QueryInfoKey(open_key)

            # Рекурсивно удаляем подклавиши
            # Мы всегда удаляем ключ с индексом 0, пока они есть
            while info[0] > 0:
                subkey_name = winreg.EnumKey(open_key, 0)
                self._delete_registry_tree(open_key, subkey_name)
                info = winreg.QueryInfoKey(open_key)

            winreg.CloseKey(open_key)
            winreg.DeleteKey(root_key, key_path)
            return True
        except FileNotFoundError:
            return True  # Уже удалено
        except Exception as e:
            raise e

    def install_context_menu(self, custom_python_path: str = None) -> Tuple[bool, str]:
        """Регистрация пункта в контекстном меню"""
        if not self.is_admin():
            self.restart_as_admin("--install-context")
            return False, "Запрошены права администратора. Проверьте открывшееся окно."

        try:
            if getattr(sys, 'frozen', False):
                exe_path = sys.executable
                command = f'"{exe_path}" --cli --path "%1"'
                icon_path = exe_path
            else:
                if custom_python_path and os.path.exists(custom_python_path):
                    python_exe = custom_python_path
                else:
                    python_exe = sys.executable

                script_path = os.path.abspath(sys.argv[0])
                if not script_path.endswith("main.py"):
                    possible = os.path.join(os.getcwd(), "main.py")
                    if os.path.exists(possible):
                        script_path = possible

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
        if not self.is_admin():
            self.restart_as_admin("--remove-context")
            return False, "Запрошены права администратора. Проверьте открывшееся окно."

        key_path = r"Directory\shell\CodeContextAI"
        try:
            self._delete_registry_tree(winreg.HKEY_CLASSES_ROOT, key_path)
            return True, "Успешно! Пункт меню удален."
        except FileNotFoundError:
            return True, "Пункт меню уже удален или не существовал."
        except Exception as e:
            # Если ошибка прав доступа, хотя is_admin() вернул True (редкий кейс)
            if "[WinError 5]" in str(e):
                return False, f"Ошибка доступа (попробуйте запустить IDE от админа): {e}"
            return False, f"Ошибка удаления из реестра: {e}"