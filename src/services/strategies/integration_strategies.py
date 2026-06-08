"""
integration_strategies.py — Стратегии интеграции с контекстным меню ОС.

Паттерн Стратегия: каждая ОС — отдельный класс, IntegrationService не знает деталей.
winreg/ctypes импортируются ТОЛЬКО внутри Windows-класса (не на уровне модуля).
"""

import os
import sys
from abc import ABC, abstractmethod
from typing import Tuple, Optional, Any


class ContextMenuStrategy(ABC):
    """Базовый интерфейс стратегии контекстного меню."""

    @abstractmethod
    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        """Устанавливает пункт контекстного меню."""

    @abstractmethod
    def remove(self) -> Tuple[bool, str]:
        """Удаляет пункт контекстного меню."""

    @abstractmethod
    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        """Добавляет команду 'codecontext' в глобальный PATH."""

    @abstractmethod
    def remove_cli(self) -> Tuple[bool, str]:
        """Удаляет команду 'codecontext' из глобального PATH."""


# ===========================================================================
# Windows
# ===========================================================================

class WindowsContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для Windows: запись в реестр HKEY_CLASSES_ROOT."""

    # winreg и ctypes импортируются только здесь — не на уровне модуля,
    # чтобы не падать с ImportError на Linux/macOS.
    def _imports(self):
        import winreg
        import ctypes
        return winreg, ctypes

    def _is_admin(self) -> bool:
        try:
            _, ctypes = self._imports()
            return bool(ctypes.windll.shell32.IsUserAnAdmin())
        except Exception:
            return False

    def _restart_as_admin(self, extra_arg: str):
        _, ctypes = self._imports()
        if getattr(sys, 'frozen', False):
            executable = sys.executable
            params = extra_arg
        else:
            executable = sys.executable
            script_path = os.path.abspath(sys.argv[0])
            params = f'"{script_path}" {extra_arg}'
        try:
            ctypes.windll.shell32.ShellExecuteW(None, "runas", executable, params, None, 0)
        except Exception as exc:
            print(f"Error elevating privileges: {exc}")

    def _delete_registry_tree(self, root_key: Any, key_path: str) -> bool:
        winreg, _ = self._imports()
        try:
            open_key = winreg.OpenKey(root_key, key_path, 0, winreg.KEY_ALL_ACCESS)
            while winreg.QueryInfoKey(open_key)[0] > 0:
                subkey_name = winreg.EnumKey(open_key, 0)
                self._delete_registry_tree(open_key, subkey_name)
            winreg.CloseKey(open_key)
            winreg.DeleteKey(root_key, key_path)
            return True
        except FileNotFoundError:
            return True
        except Exception as exc:
            raise exc

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        if not self._is_admin():
            self._restart_as_admin("--install-context")
            return False, "🛡 Запрошены права администратора. Применится в фоновом режиме."

        winreg, _ = self._imports()

        try:
            if getattr(sys, 'frozen', False):
                exe_path = sys.executable
                command_base = f'"{exe_path}" --cli --path'
                gui_command = f'"{exe_path}" --path'
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

                command_base = f'"{python_exe}" "{script_path}" --cli --path'
                gui_command = f'"{python_exe}" "{script_path}" --path'
                icon_path = python_exe

            # Универсальная функция-помощник для создания вложенного меню
            def _add_menu(root_key, path, target_arg):
                key = winreg.CreateKey(root_key, path)
                try:
                    winreg.DeleteValue(key, "")
                except OSError:
                    pass
                winreg.SetValueEx(key, "MUIVerb", 0, winreg.REG_SZ, "CodeContext AI")
                winreg.SetValueEx(key, "Icon", 0, winreg.REG_SZ, icon_path)
                winreg.SetValueEx(key, "SubCommands", 0, winreg.REG_SZ, "")

                sub_shell = winreg.CreateKey(key, "shell")

                # 1. Запуск GUI
                sub_gui = winreg.CreateKey(sub_shell, "cmd_gui")
                winreg.SetValue(sub_gui, "", winreg.REG_SZ, "📂 Открыть UI")
                winreg.SetValueEx(sub_gui, "Icon", 0, winreg.REG_SZ, icon_path)
                cmd_gui = winreg.CreateKey(sub_gui, "command")
                winreg.SetValue(cmd_gui, "", winreg.REG_SZ, f'{gui_command} "{target_arg}"')
                winreg.CloseKey(cmd_gui)
                winreg.CloseKey(sub_gui)

                # 2. Быстрые команды (Без UI)
                commands = [
                    ("cmd1", "📋 Скопировать (Default)", f'--silent'),
                    ("cmd2", "📦 Скопировать (Deep Deps)", f'--mode deep --silent'),
                    ("cmd3", "📄 Скопировать как XML", f'--format xml --silent'),
                    ("cmd4", "☠️ Скопировать Skeleton", f'--skeleton true --silent')
                ]
                for k_name, k_label, flags in commands:
                    sub = winreg.CreateKey(sub_shell, k_name)
                    winreg.SetValue(sub, "", winreg.REG_SZ, k_label)
                    winreg.SetValueEx(sub, "Icon", 0, winreg.REG_SZ, icon_path)
                    cmd = winreg.CreateKey(sub, "command")
                    winreg.SetValue(cmd, "", winreg.REG_SZ, f'{command_base} "{target_arg}" {flags}')
                    winreg.CloseKey(cmd)
                    winreg.CloseKey(sub)

                winreg.CloseKey(sub_shell)
                winreg.CloseKey(key)

            # 1. Применяем для папок (клик ПКМ по самой папке)
            _add_menu(winreg.HKEY_CLASSES_ROOT, r"Directory\shell\CodeContextAI", "%1")

            # 2. Применяем для фона папки (клик ПКМ по пустому месту внутри папки)
            # В Windows для передачи пути фона используется аргумент "%V"
            _add_menu(winreg.HKEY_CLASSES_ROOT, r"Directory\Background\shell\CodeContextAI", "%V")

            # 3. Применяем для файлов (клик ПКМ по файлу)
            _add_menu(winreg.HKEY_CLASSES_ROOT, r"*\shell\CodeContextAI", "%1")

            return True, "Успешно! Пункты добавлены для папок и файлов."
        except Exception as exc:
            return False, f"Ошибка записи в реестр: {exc}"

    def remove(self) -> Tuple[bool, str]:
        if not self._is_admin():
            self._restart_as_admin("--remove-context")
            return False, "🛡 Запрошены права администратора. Будет удалено в фоновом режиме."

        winreg, _ = self._imports()
        err_msg = ""

        # Очищаем все три ветки
        for path in (
            r"Directory\shell\CodeContextAI",
            r"Directory\Background\shell\CodeContextAI",
            r"*\shell\CodeContextAI"
        ):
            try:
                self._delete_registry_tree(winreg.HKEY_CLASSES_ROOT, path)
            except Exception as exc:
                if not isinstance(exc, FileNotFoundError):
                    err_msg += str(exc)

        if err_msg:
            if "[WinError 5]" in err_msg:
                return False, f"Ошибка доступа (запустите IDE от администратора): {err_msg}"
            return False, f"Ошибка удаления из реестра: {err_msg}"

        return True, "Успешно! Пункты меню удалены."

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            winreg, ctypes = self._imports()
            is_frozen = getattr(sys, 'frozen', False)
            exe_path = sys.executable
            python_exe = custom_python_path if custom_python_path and os.path.exists(custom_python_path) else sys.executable
            script_path = os.path.abspath(sys.argv[0])

            from ...utils.config import get_app_data_dir
            bin_dir = os.path.join(get_app_data_dir(), "bin")
            os.makedirs(bin_dir, exist_ok=True)

            bat_path = os.path.join(bin_dir, "codecontext.bat")
            with open(bat_path, "w", encoding="utf-8-sig") as f:
                f.write('@echo off\n')
                f.write('chcp 65001 > nul\n')
                if is_frozen:
                    f.write(f'"{exe_path}" %*\n')
                else:
                    f.write(f'"{python_exe}" "{script_path}" %*\n')

            key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_ALL_ACCESS)
            try:
                path_val, _ = winreg.QueryValueEx(key, "Path")
            except FileNotFoundError:
                path_val = ""

            if bin_dir not in path_val.split(os.pathsep):
                new_path = path_val + (os.pathsep if path_val and not path_val.endswith(os.pathsep) else "") + bin_dir
                winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, new_path)

                HWND_BROADCAST = 0xFFFF
                WM_SETTINGCHANGE = 0x001A
                SMTO_ABORTIFHUNG = 0x0002
                ctypes.windll.user32.SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment", SMTO_ABORTIFHUNG, 5000, None)

            winreg.CloseKey(key)
            return True, "Успешно! Команда 'codecontext' добавлена в PATH.\n(Перезапустите терминал для применения)"
        except Exception as e:
            return False, f"Ошибка добавления в PATH: {e}"

    def remove_cli(self) -> Tuple[bool, str]:
        try:
            winreg, ctypes = self._imports()
            from ...utils.config import get_app_data_dir
            import shutil

            bin_dir = os.path.join(get_app_data_dir(), "bin")
            if os.path.exists(bin_dir):
                shutil.rmtree(bin_dir, ignore_errors=True)

            key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_ALL_ACCESS)
            try:
                path_val, _ = winreg.QueryValueEx(key, "Path")
                paths = [p for p in path_val.split(os.pathsep) if p and p != bin_dir]
                new_path = os.pathsep.join(paths)
                winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, new_path)

                HWND_BROADCAST = 0xFFFF
                WM_SETTINGCHANGE = 0x001A
                SMTO_ABORTIFHUNG = 0x0002
                ctypes.windll.user32.SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment", SMTO_ABORTIFHUNG, 5000, None)
            except FileNotFoundError:
                pass
            finally:
                winreg.CloseKey(key)
            return True, "Успешно! Команда 'codecontext' удалена из PATH."
        except Exception as e:
            return False, f"Ошибка удаления: {e}"


# ===========================================================================
# Linux
# ===========================================================================

class LinuxContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для Linux: .desktop файлы для KDE/GTK файловых менеджеров."""

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            python_exe = custom_python_path or sys.executable
            script_path = os.path.abspath(sys.argv[0])

            kde_dir = os.path.expanduser("~/.local/share/kio/servicemenus")
            os.makedirs(kde_dir, exist_ok=True)
            with open(os.path.join(kde_dir, "codecontext_ai.desktop"), "w") as f:
                f.write(
                    f"[Desktop Entry]\nType=Service\nServiceTypes=KonqPopupMenu/Plugin\n"
                    f"MimeType=all/allfiles;inode/directory;\n"
                    f"Actions=OpenGUI;CopyXML;CopySkeleton;\n"
                    f"X-KDE-Priority=TopLevel\n"
                    f"X-KDE-Submenu=CodeContext AI\n"
                    f"Icon=utilities-terminal\n\n"

                    f"[Desktop Action OpenGUI]\nName=📂 Открыть UI\n"
                    f"Icon=utilities-terminal\nExec={python_exe} {script_path} --path %f\n\n"

                    f"[Desktop Action CopyXML]\nName=📄 Скопировать как XML\n"
                    f"Icon=edit-copy\nExec={python_exe} {script_path} --cli --path %f --format xml --silent\n\n"

                    f"[Desktop Action CopySkeleton]\nName=☠️ Скопировать Skeleton\n"
                    f"Icon=edit-copy\nExec={python_exe} {script_path} --cli --path %f --skeleton true --silent\n"
                )
            return True, "Успешно! Контекстное меню добавлено (KDE/GTK)."
        except Exception as exc:
            return False, f"Ошибка установки в Linux: {exc}"

    def remove(self) -> Tuple[bool, str]:
        try:
            for path in (
                "~/.local/share/file-manager/actions/codecontext_ai.desktop",
                "~/.local/share/kio/servicemenus/codecontext_ai.desktop",
            ):
                full = os.path.expanduser(path)
                if os.path.exists(full):
                    os.remove(full)
            return True, "Успешно! Контекстное меню удалено."
        except Exception as exc:
            return False, f"Ошибка удаления в Linux: {exc}"

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            is_frozen = getattr(sys, 'frozen', False)
            exe_path = sys.executable
            python_exe = custom_python_path or sys.executable
            script_path = os.path.abspath(sys.argv[0])

            bin_dir = os.path.expanduser("~/.local/bin")
            os.makedirs(bin_dir, exist_ok=True)

            sh_path = os.path.join(bin_dir, "codecontext")
            with open(sh_path, "w", encoding="utf-8") as f:
                if is_frozen:
                    f.write(f'#!/bin/bash\nexec "{exe_path}" "$@"\n')
                else:
                    f.write(f'#!/bin/bash\nexec "{python_exe}" "{script_path}" "$@"\n')

            os.chmod(sh_path, 0o755)
            return True, "Успешно! Скрипт 'codecontext' добавлен в ~/.local/bin.\n(Убедитесь, что ~/.local/bin есть в вашем PATH)"
        except Exception as e:
            return False, f"Ошибка добавления в PATH: {e}"

    def remove_cli(self) -> Tuple[bool, str]:
        try:
            sh_path = os.path.expanduser("~/.local/bin/codecontext")
            if os.path.exists(sh_path):
                os.remove(sh_path)
            return True, "Успешно! Команда 'codecontext' удалена."
        except Exception as e:
            return False, f"Ошибка удаления: {e}"


# ===========================================================================
# macOS
# ===========================================================================

class MacOSContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для macOS (Automator/Quick Actions)."""

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        return LinuxContextMenuStrategy().install_cli(custom_python_path)

    def remove_cli(self) -> Tuple[bool, str]:
        return LinuxContextMenuStrategy().remove_cli()

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        return True, "Контекстное меню macOS реализовано через Automator (пока поддерживает только базовые функции)."

    def remove(self) -> Tuple[bool, str]:
        return True, "Контекстное меню macOS удалено."