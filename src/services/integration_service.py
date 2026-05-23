import sys
import os
import winreg
import ctypes
from typing import Tuple, Any, Optional


class IntegrationService:
    """Сервис для интеграции с Windows (Реестр, Права администратора)"""

    @staticmethod
    def is_admin() -> bool:
        """Проверяет, запущен ли процесс с правами администратора"""
        try:
            is_admin_func = getattr(ctypes.windll.shell32, "IsUserAnAdmin")
            return bool(is_admin_func())
        except (AttributeError, OSError, Exception):
            return False

    @staticmethod
    def restart_as_admin(extra_arg: str):
        if getattr(sys, 'frozen', False):
            executable = sys.executable
            params = extra_arg
        else:
            executable = sys.executable
            script_path = os.path.abspath(sys.argv[0])
            params = f'"{script_path}" {extra_arg}'

        try:
            shell_execute = getattr(ctypes.windll.shell32, "ShellExecuteW")
            shell_execute(None, "runas", executable, params, None, 1)
        except Exception as e:
            print(f"Error elevating privileges: {e}")

    def _delete_registry_tree(self, root_key: Any, key_path: str) -> bool:
        try:
            open_key = winreg.OpenKey(root_key, key_path, 0, winreg.KEY_ALL_ACCESS)
            # Цикл удаляет подпапки, пока они существуют
            while winreg.QueryInfoKey(open_key)[0] > 0:
                subkey_name = winreg.EnumKey(open_key, 0)
                self._delete_registry_tree(open_key, subkey_name)
            winreg.CloseKey(open_key)
            winreg.DeleteKey(root_key, key_path)
            return True
        except FileNotFoundError:
            return True
        except Exception as e:
            raise e

    def install_context_menu(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        if not self.is_admin():
            self.restart_as_admin("--install-context")
            return False, "Запрошены права администратора. Проверьте открывшееся окно."

        try:
            if getattr(sys, 'frozen', False):
                exe_path = sys.executable
                command_base = f'"{exe_path}" --cli --path "%1"'
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

                command_base = f'"{python_exe}" "{script_path}" --cli --path "%1"'
                icon_path = python_exe

            key_path_dir = r"Directory\shell\CodeContextAI"
            key_dir = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, key_path_dir)
            winreg.SetValue(key_dir, "", winreg.REG_SZ, "Scan with CodeContext AI")
            winreg.SetValueEx(key_dir, "Icon", 0, winreg.REG_SZ, icon_path)
            command_key_dir = winreg.CreateKey(key_dir, "command")
            winreg.SetValue(command_key_dir, "", winreg.REG_SZ, command_base)
            winreg.CloseKey(command_key_dir)
            winreg.CloseKey(key_dir)

            key_path_file = r"*\shell\CodeContextAI"
            key_file = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, key_path_file)
            winreg.SetValueEx(key_file, "MUIVerb", 0, winreg.REG_SZ, "CodeContext AI")
            winreg.SetValueEx(key_file, "Icon", 0, winreg.REG_SZ, icon_path)
            winreg.SetValueEx(key_file, "SubCommands", 0, winreg.REG_SZ, "")

            sub_shell = winreg.CreateKey(key_file, "shell")

            cmd1 = winreg.CreateKey(sub_shell, "cmd1")
            winreg.SetValue(cmd1, "", winreg.REG_SZ, "Scan File Only (No Deps)")
            winreg.SetValueEx(cmd1, "Icon", 0, winreg.REG_SZ, icon_path)
            c1 = winreg.CreateKey(cmd1, "command")
            winreg.SetValue(c1, "", winreg.REG_SZ, command_base + ' --mode default')
            winreg.CloseKey(c1)
            winreg.CloseKey(cmd1)

            cmd2 = winreg.CreateKey(sub_shell, "cmd2")
            winreg.SetValue(cmd2, "", winreg.REG_SZ, "Scan File + Shallow Deps")
            winreg.SetValueEx(cmd2, "Icon", 0, winreg.REG_SZ, icon_path)
            c2 = winreg.CreateKey(cmd2, "command")
            winreg.SetValue(c2, "", winreg.REG_SZ, command_base + ' --mode shallow')
            winreg.CloseKey(c2)
            winreg.CloseKey(cmd2)

            cmd3 = winreg.CreateKey(sub_shell, "cmd3")
            winreg.SetValue(cmd3, "", winreg.REG_SZ, "Scan File + Deep Deps")
            winreg.SetValueEx(cmd3, "Icon", 0, winreg.REG_SZ, icon_path)
            c3 = winreg.CreateKey(cmd3, "command")
            winreg.SetValue(c3, "", winreg.REG_SZ, command_base + ' --mode deep')
            winreg.CloseKey(c3)
            winreg.CloseKey(cmd3)

            winreg.CloseKey(sub_shell)
            winreg.CloseKey(key_file)

            return True, "Успешно! Пункты добавлены для папок и файлов."
        except Exception as e:
            return False, f"Ошибка записи в реестр: {e}"

    def remove_context_menu(self) -> Tuple[bool, str]:
        if not self.is_admin():
            self.restart_as_admin("--remove-context")
            return False, "Запрошены права администратора. Проверьте открывшееся окно."

        err_msg = ""
        try:
            self._delete_registry_tree(winreg.HKEY_CLASSES_ROOT, r"Directory\shell\CodeContextAI")
        except Exception as e:
            if not isinstance(e, FileNotFoundError): err_msg += str(e)

        try:
            self._delete_registry_tree(winreg.HKEY_CLASSES_ROOT, r"*\shell\CodeContextAI")
        except Exception as e:
            if not isinstance(e, FileNotFoundError): err_msg += str(e)

        if err_msg:
            if "[WinError 5]" in err_msg:
                return False, f"Ошибка доступа (попробуйте запустить IDE от админа): {err_msg}"
            return False, f"Ошибка удаления из реестра: {err_msg}"

        return True, "Успешно! Пункты меню удалены."