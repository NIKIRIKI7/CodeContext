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

            # Для папок
            key_dir = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, r"Directory\shell\CodeContextAI")
            winreg.SetValue(key_dir, "", winreg.REG_SZ, "Scan with CodeContext AI")
            winreg.SetValueEx(key_dir, "Icon", 0, winreg.REG_SZ, icon_path)
            cmd_dir = winreg.CreateKey(key_dir, "command")
            winreg.SetValue(cmd_dir, "", winreg.REG_SZ, command_base)
            winreg.CloseKey(cmd_dir)
            winreg.CloseKey(key_dir)

            # Для файлов (с подменю)
            key_file = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, r"*\shell\CodeContextAI")
            winreg.SetValueEx(key_file, "MUIVerb", 0, winreg.REG_SZ, "CodeContext AI")
            winreg.SetValueEx(key_file, "Icon", 0, winreg.REG_SZ, icon_path)
            winreg.SetValueEx(key_file, "SubCommands", 0, winreg.REG_SZ, "")
            sub_shell = winreg.CreateKey(key_file, "shell")

            for key_name, label, mode in [
                ("cmd1", "Scan File Only (No Deps)", "default"),
                ("cmd2", "Scan File + Shallow Deps", "shallow"),
                ("cmd3", "Scan File + Deep Deps",    "deep"),
            ]:
                sub = winreg.CreateKey(sub_shell, key_name)
                winreg.SetValue(sub, "", winreg.REG_SZ, label)
                winreg.SetValueEx(sub, "Icon", 0, winreg.REG_SZ, icon_path)
                cmd = winreg.CreateKey(sub, "command")
                winreg.SetValue(cmd, "", winreg.REG_SZ, f"{command_base} --mode {mode}")
                winreg.CloseKey(cmd)
                winreg.CloseKey(sub)

            winreg.CloseKey(sub_shell)
            winreg.CloseKey(key_file)
            return True, "Успешно! Пункты добавлены для папок и файлов."

        except Exception as exc:
            return False, f"Ошибка записи в реестр: {exc}"

    def remove(self) -> Tuple[bool, str]:
        if not self._is_admin():
            self._restart_as_admin("--remove-context")
            return False, "🛡 Запрошены права администратора. Будет удалено в фоновом режиме."

        winreg, _ = self._imports()
        err_msg = ""
        for path in (r"Directory\shell\CodeContextAI", r"*\shell\CodeContextAI"):
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


# ===========================================================================
# Linux
# ===========================================================================

class LinuxContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для Linux: .desktop файлы для KDE/GTK файловых менеджеров."""

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            python_exe = custom_python_path or sys.executable
            script_path = os.path.abspath(sys.argv[0])

            # GTK (Nautilus, Nemo, Caja)
            action_dir = os.path.expanduser("~/.local/share/file-manager/actions")
            os.makedirs(action_dir, exist_ok=True)
            with open(os.path.join(action_dir, "codecontext_ai.desktop"), "w") as f:
                f.write(
                    f"[Desktop Entry]\nType=Action\nName=Scan with CodeContext AI\n"
                    f"Icon=utilities-terminal\nProfiles=profile-zero;\n\n"
                    f"[X-Action-Profile profile-zero]\n"
                    f"Exec={python_exe} {script_path} --cli --path %f\n"
                    f"Name=Default profile\n"
                )

            # KDE (Dolphin)
            kde_dir = os.path.expanduser("~/.local/share/kio/servicemenus")
            os.makedirs(kde_dir, exist_ok=True)
            with open(os.path.join(kde_dir, "codecontext_ai.desktop"), "w") as f:
                f.write(
                    f"[Desktop Entry]\nType=Service\nServiceTypes=KonqPopupMenu/Plugin\n"
                    f"MimeType=all/allfiles;inode/directory;\nActions=ScanCodeContext;\n"
                    f"X-KDE-Priority=TopLevel\n\n"
                    f"[Desktop Action ScanCodeContext]\nName=Scan with CodeContext AI\n"
                    f"Icon=utilities-terminal\nExec={python_exe} {script_path} --cli --path %f\n"
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


# ===========================================================================
# macOS
# ===========================================================================

class MacOSContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для macOS (Automator/Quick Actions — в разработке)."""

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        return False, "Интеграция с Finder (macOS) пока в разработке."

    def remove(self) -> Tuple[bool, str]:
        return False, "Интеграция с Finder (macOS) пока в разработке."