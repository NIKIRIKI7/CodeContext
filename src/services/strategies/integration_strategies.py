"""
integration_strategies.py — Стратегии интеграции с контекстным меню ОС.

Паттерн Стратегия: каждая ОС — отдельный класс, IntegrationService не знает деталей.
winreg/ctypes импортируются ТОЛЬКО внутри Windows-класса (не на уровне модуля).
"""

import os
import plistlib
import shutil
import sys
import uuid
from abc import ABC, abstractmethod
from typing import Tuple, Optional, Any

from src.i18n import tr


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
                winreg.SetValue(sub_gui, "", winreg.REG_SZ, tr("integration_strategies.windows.menu_open_ui"))
                winreg.SetValueEx(sub_gui, "Icon", 0, winreg.REG_SZ, icon_path)
                cmd_gui = winreg.CreateKey(sub_gui, "command")
                winreg.SetValue(cmd_gui, "", winreg.REG_SZ, f'{gui_command} "{target_arg}"')
                winreg.CloseKey(cmd_gui)
                winreg.CloseKey(sub_gui)

                # 2. Быстрые команды (Без UI)
                commands = [
                    ("cmd1", tr("integration_strategies.windows.menu_copy_default"), f'--silent'),
                    ("cmd2", tr("integration_strategies.windows.menu_copy_deep"), f'--mode deep --silent'),
                    ("cmd3", tr("integration_strategies.windows.menu_copy_xml"), f'--format xml --silent'),
                    ("cmd4", tr("integration_strategies.windows.menu_copy_skeleton"), f'--skeleton true --silent')
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

            # ponytail: Use HKCU to install without admin privileges
            _add_menu(winreg.HKEY_CURRENT_USER, r"Software\Classes\Directory\shell\CodeContextAI", "%1")
            _add_menu(winreg.HKEY_CURRENT_USER, r"Software\Classes\Directory\Background\shell\CodeContextAI", "%V")
            _add_menu(winreg.HKEY_CURRENT_USER, r"Software\Classes\*\shell\CodeContextAI", "%1")

            return True, tr("integration_strategies.windows.install_success")
        except Exception as exc:
            return False, tr("integration_strategies.windows.install_error", error=exc)

    def remove(self) -> Tuple[bool, str]:
        winreg, _ = self._imports()
        err_msg = ""

        # ponytail: Delete from HKCU (no admin required)
        for path in (
            r"Software\Classes\Directory\shell\CodeContextAI",
            r"Software\Classes\Directory\Background\shell\CodeContextAI",
            r"Software\Classes\*\shell\CodeContextAI"
        ):
            try:
                self._delete_registry_tree(winreg.HKEY_CURRENT_USER, path)
            except Exception as exc:
                if not isinstance(exc, FileNotFoundError):
                    err_msg += str(exc)

        # Legacy cleanup for old HKCR installs. Ignores access denied errors silently.
        for path in (r"Directory\shell\CodeContextAI", r"Directory\Background\shell\CodeContextAI", r"*\shell\CodeContextAI"):
            try: self._delete_registry_tree(winreg.HKEY_CLASSES_ROOT, path)
            except Exception: pass

        if err_msg:
            return False, tr("integration_strategies.windows.remove_registry_error", error=err_msg)

        return True, tr("integration_strategies.windows.remove_success")

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            winreg, ctypes = self._imports()
            is_frozen = getattr(sys, 'frozen', False)
            exe_path = sys.executable
            python_exe = custom_python_path if custom_python_path and os.path.exists(custom_python_path) else sys.executable
            script_path = os.path.abspath(sys.argv[0])

            from ...utils.config import get_app_data_dir
            bin_dir = os.path.join(get_app_data_dir(), "bin")

            def _get_short_path(long_path: str) -> str:
                buf = ctypes.create_unicode_buffer(260)
                ctypes.windll.kernel32.GetShortPathNameW(long_path, buf, 260)
                return buf.value or long_path

            short_bin = _get_short_path(bin_dir)
            os.makedirs(short_bin, exist_ok=True)

            bat_path = os.path.join(short_bin, "codecontext.bat")
            with open(bat_path, "w", encoding="utf-8") as f:
                f.write('@echo off\n')
                f.write('chcp 65001 > nul\n')
                f.write('set PYTHONIOENCODING=utf-8\n')
                if is_frozen:
                    f.write(f'"{_get_short_path(exe_path)}" %*\n')
                else:
                    f.write(f'"{_get_short_path(python_exe)}" "{_get_short_path(script_path)}" %*\n')

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
            return True, tr("integration_strategies.windows.cli_install_success")
        except Exception as e:
            return False, tr("integration_strategies.windows.cli_install_error", error=e)

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
            return True, tr("integration_strategies.windows.cli_remove_success")
        except Exception as e:
            return False, tr("integration_strategies.windows.cli_remove_error", error=e)


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

                    f"[Desktop Action OpenGUI]\nName={tr('integration_strategies.linux.menu_open_ui')}\n"
                    f"Icon=utilities-terminal\nExec={python_exe} {script_path} --path %f\n\n"

                    f"[Desktop Action CopyXML]\nName={tr('integration_strategies.linux.menu_copy_xml')}\n"
                    f"Icon=edit-copy\nExec={python_exe} {script_path} --cli --path %f --format xml --silent\n\n"

                    f"[Desktop Action CopySkeleton]\nName={tr('integration_strategies.linux.menu_copy_skeleton')}\n"
                    f"Icon=edit-copy\nExec={python_exe} {script_path} --cli --path %f --skeleton true --silent\n"
                )
            return True, tr("integration_strategies.linux.install_success")
        except Exception as exc:
            return False, tr("integration_strategies.linux.install_error", error=exc)

    def remove(self) -> Tuple[bool, str]:
        try:
            for path in (
                "~/.local/share/file-manager/actions/codecontext_ai.desktop",
                "~/.local/share/kio/servicemenus/codecontext_ai.desktop",
            ):
                full = os.path.expanduser(path)
                if os.path.exists(full):
                    os.remove(full)
            return True, tr("integration_strategies.linux.remove_success")
        except Exception as exc:
            return False, tr("integration_strategies.linux.remove_error", error=exc)

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
            return True, tr("integration_strategies.linux.cli_install_success")
        except Exception as e:
            return False, tr("integration_strategies.linux.cli_install_error", error=e)

    def remove_cli(self) -> Tuple[bool, str]:
        try:
            sh_path = os.path.expanduser("~/.local/bin/codecontext")
            if os.path.exists(sh_path):
                os.remove(sh_path)
            return True, tr("integration_strategies.linux.cli_remove_success")
        except Exception as e:
            return False, tr("integration_strategies.linux.cli_remove_error", error=e)


# ===========================================================================
# macOS
# ===========================================================================

class MacOSContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для macOS: Automator Quick Actions в ~/Library/Services/."""

    # ------------------------------------------------------------------
    # CLI (shell-скрипт в ~/.local/bin) — та же логика, что у Linux
    # ------------------------------------------------------------------

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        return LinuxContextMenuStrategy().install_cli(custom_python_path)

    def remove_cli(self) -> Tuple[bool, str]:
        return LinuxContextMenuStrategy().remove_cli()

    # ------------------------------------------------------------------
    # Помощники
    # ------------------------------------------------------------------

    def _get_commands(self, custom_python_path: Optional[str] = None) -> dict[str, str]:
        """
        Возвращает словарь {имя пункта меню → shell-команда}.
        Команда получает путь через $1 и запускается в фоне.
        """
        if getattr(sys, 'frozen', False):
            exe = sys.executable
            return {
                "CodeContext AI":
                    f'"{exe}" --path "$1" > /dev/null 2>&1 &',
                "CodeContext AI (XML)":
                    f'"{exe}" --cli --path "$1" --format xml --silent > /dev/null 2>&1 &',
                "CodeContext AI (Skeleton)":
                    f'"{exe}" --cli --path "$1" --skeleton true --silent > /dev/null 2>&1 &',
            }

        python = custom_python_path or sys.executable
        script = os.path.abspath(sys.argv[0])
        if not script.endswith("main.py"):
            candidate = os.path.join(os.getcwd(), "main.py")
            if os.path.exists(candidate):
                script = candidate

        return {
            "CodeContext AI":
                f'"{python}" "{script}" --path "$1" > /dev/null 2>&1 &',
            "CodeContext AI (XML)":
                f'"{python}" "{script}" --cli --path "$1" --format xml --silent > /dev/null 2>&1 &',
            "CodeContext AI (Skeleton)":
                f'"{python}" "{script}" --cli --path "$1" --skeleton true --silent > /dev/null 2>&1 &',
        }

    def _create_workflow(self, name: str, command: str) -> None:
        services_dir = os.path.expanduser("~/Library/Services")
        os.makedirs(services_dir, exist_ok=True)
        workflow_path = os.path.join(services_dir, f"{name}.workflow")
        if os.path.exists(workflow_path):
            shutil.rmtree(workflow_path)
        contents_dir = os.path.join(workflow_path, "Contents")
        os.makedirs(contents_dir, exist_ok=True)

        bundle_id = f"com.codecontext.ai.{uuid.uuid4().hex}"
        uid = lambda: str(uuid.uuid4()).upper()

        plistlib.dump({
            "CFBundleIdentifier": bundle_id,
            "CFBundleName": name,
            "CFBundlePackageType": "BNDL",
            "CFBundleShortVersionString": "1.0",
            "CFBundleSignature": "????",
            "CFBundleVersion": "1.0",
            "NSServices": [{
                "NSMenuItem": {"default": name},
                "NSMessage": "runWorkflowAsService",
                "NSSendFileTypes": ["public.folder", "public.directory"],
            }],
        }, open(os.path.join(contents_dir, "Info.plist"), "wb"))

        plistlib.dump({
            "AMApplicationBuild": "523",
            "AMApplicationVersion": "2.10",
            "AMDocumentVersion": "2",
            "actions": [{
                "action": {
                    "AMAccepts": {"Container": "List", "Optional": True, "Types": ["com.apple.cocoa.string"]},
                    "AMActionVersion": "2.0.3",
                    "AMApplication": ["Automator"],
                    "AMParameterProperties": {
                        "COMMANDString": {}, "CheckedForUserDefaultShell": {},
                        "inputMethod": {}, "shell": {}, "source": {},
                    },
                    "AMProvides": {"Container": "List", "Types": ["com.apple.cocoa.string"]},
                    "ActionBundlePath": "/System/Library/Automator/Run Shell Script.action",
                    "ActionName": "Run Shell Script",
                    "ActionParameters": {
                        "COMMANDString": command,
                        "CheckedForUserDefaultShell": True,
                        "inputMethod": 1,
                        "shell": "/bin/bash",
                        "source": "",
                    },
                    "BundleIdentifier": "com.apple.RunShellScript",
                    "CFBundleVersion": "2.0.3",
                    "CanShowSelectedItemsWhenRun": False,
                    "CanShowWhenRun": True,
                    "Category": ["AMCategoryUtilities"],
                    "Class Name": "RunShellScriptAction",
                    "InputUUID": uid(),
                    "Keywords": ["Shell", "Script", "Command", "Run", "Unix"],
                    "OutputUUID": uid(),
                    "UUID": uid(),
                    "UnlocalizedApplications": ["Automator"],
                    "arguments": {
                        "0": {"default value": 0, "name": "inputMethod", "required": "0", "type": "0", "uuid": "0"},
                        "1": {"default value": False, "name": "CheckedForUserDefaultShell", "required": "0", "type": "0", "uuid": "1"},
                        "2": {"default value": "", "name": "source", "required": "0", "type": "0", "uuid": "2"},
                        "3": {"default value": "", "name": "COMMANDString", "required": "0", "type": "0", "uuid": "3"},
                        "4": {"default value": "/bin/sh", "name": "shell", "required": "0", "type": "0", "uuid": "4"},
                    },
                    "isViewVisible": 1,
                    "location": "309.000000:305.000000",
                },
                "isViewVisible": 1,
            }],
            "connectors": {},
            "workflowMetaData": {
                "applicationBundleIDsByPath": {},
                "applicationPaths": [],
                "inputTypeIdentifier": "com.apple.Automator.fileSystemObject.folder",
                "outputTypeIdentifier": "com.apple.Automator.nothing",
                "presentationMode": 15,
                "processesInput": 0,
                "systemImageName": "NSActionTemplate",
                "useAutomaticInputType": 0,
                "workflowTypeIdentifier": "com.apple.Automator.servicesMenu",
            },
        }, open(os.path.join(contents_dir, "document.wflow"), "wb"))

    # ------------------------------------------------------------------
    # install / remove
    # ------------------------------------------------------------------

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            import subprocess

            commands = self._get_commands(custom_python_path)
            for name, cmd in commands.items():
                self._create_workflow(name, cmd)

            # Сброс кеша Service-ов, чтобы Finder увидел новый пункт сразу
            try:
                subprocess.run(
                    ["/System/Library/CoreServices/pbs", "-flush"],
                    check=False,
                )
            except Exception:
                pass

            return True, tr(
                "integration_strategies.macos.install_success",
                default="Context menu installed successfully.",
            )
        except Exception as exc:
            return False, tr(
                "integration_strategies.macos.install_error",
                default=f"Error installing context menu: {exc}",
                error=exc,
            )

    def remove(self) -> Tuple[bool, str]:
        try:
            import subprocess

            services_dir = os.path.expanduser("~/Library/Services")
            commands = self._get_commands()
            for name in commands.keys():
                workflow_path = os.path.join(services_dir, f"{name}.workflow")
                if os.path.exists(workflow_path):
                    shutil.rmtree(workflow_path)

            try:
                subprocess.run(
                    ["/System/Library/CoreServices/pbs", "-flush"],
                    check=False,
                )
            except Exception:
                pass

            return True, tr(
                "integration_strategies.macos.remove_success",
                default="Context menu removed successfully.",
            )
        except Exception as exc:
            return False, tr(
                "integration_strategies.macos.remove_error",
                default=f"Error removing context menu: {exc}",
                error=exc,
            )