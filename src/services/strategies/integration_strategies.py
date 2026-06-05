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
                winreg.SetValue(key, "", winreg.REG_SZ, "CodeContext AI")
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

                # 2. Опции быстрого сканирования (Без UI)
                for k_name, k_label, mode in [
                    ("cmd1", "📋 Скопировать (Без зависимостей)", "default"),
                    ("cmd2", "📦 Скопировать (Shallow зависимости)", "shallow"),
                    ("cmd3", "📦 Скопировать (Deep зависимости)", "deep"),
                ]:
                    sub = winreg.CreateKey(sub_shell, k_name)
                    winreg.SetValue(sub, "", winreg.REG_SZ, k_label)
                    winreg.SetValueEx(sub, "Icon", 0, winreg.REG_SZ, icon_path)
                    cmd = winreg.CreateKey(sub, "command")
                    winreg.SetValue(cmd, "", winreg.REG_SZ, f'{command_base} "{target_arg}" --mode {mode} --silent')
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


# ===========================================================================
# Linux
# ===========================================================================

class LinuxContextMenuStrategy(ContextMenuStrategy):
    """Стратегия для Linux: .desktop файлы для KDE/GTK файловых менеджеров."""

    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            python_exe = custom_python_path or sys.executable
            script_path = os.path.abspath(sys.argv[0])

            action_dir = os.path.expanduser("~/.local/share/file-manager/actions")
            os.makedirs(action_dir, exist_ok=True)
            with open(os.path.join(action_dir, "codecontext_ai.desktop"), "w") as f:
                f.write(
                    f"[Desktop Entry]\nType=Action\nName=Открыть в CodeContext AI\n"
                    f"Icon=utilities-terminal\nProfiles=profile-zero;\n\n"
                    f"[X-Action-Profile profile-zero]\n"
                    f"Exec={python_exe} {script_path} --path %f\n"
                    f"Name=Default profile\n"
                )

            kde_dir = os.path.expanduser("~/.local/share/kio/servicemenus")
            os.makedirs(kde_dir, exist_ok=True)
            with open(os.path.join(kde_dir, "codecontext_ai.desktop"), "w") as f:
                f.write(
                    f"[Desktop Entry]\nType=Service\nServiceTypes=KonqPopupMenu/Plugin\n"
                    f"MimeType=all/allfiles;inode/directory;\nActions=ScanCodeContext;\n"
                    f"X-KDE-Priority=TopLevel\n\n"
                    f"[Desktop Action ScanCodeContext]\nName=Открыть в CodeContext AI\n"
                    f"Icon=utilities-terminal\nExec={python_exe} {script_path} --path %f\n"
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
    """Стратегия для macOS (Automator/Quick Actions)."""
    def install(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        import os, sys
        python_exe = custom_python_path or sys.executable
        script_path = os.path.abspath(sys.argv[0])
        services_dir = os.path.expanduser("~/Library/Services")
        os.makedirs(services_dir, exist_ok=True)
        workflow_dir = os.path.join(services_dir, "CodeContextAI.workflow")
        os.makedirs(os.path.join(workflow_dir, "Contents"), exist_ok=True)

        info_plist = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.codecontext.workflow</string>
    <key>CFBundleName</key>
    <string>Открыть в CodeContext AI</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Открыть в CodeContext AI</string>
            </dict>
            <key>NSMessage</key>
            <string>runWorkflowAsService</string>
            <key>NSRequiredContext</key>
            <dict>
                <key>NSTextContent</key>
                <string>FilePath</string>
            </dict>
            <key>NSSendTypes</key>
            <array>
                <string>public.item</string>
            </array>
        </dict>
    </array>
</dict>
</plist>"""

        script_content = f'''for f in "$@"
do
    "{python_exe}" "{script_path}" --path "$f"
done'''

        document_wflow = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AMApplicationBuild</key>
    <string>523</string>
    <key>AMApplicationVersion</key>
    <string>2.10</string>
    <key>AMDocumentVersion</key>
    <string>2</string>
    <key>actions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMAccepts</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Optional</key>
                    <true/>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>AMActionVersion</key>
                <string>2.0.3</string>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>AMParameterProperties</key>
                <dict>
                    <key>COMMANDString</key>
                    <dict/>
                    <key>CheckedForUserDefaultShell</key>
                    <dict/>
                    <key>inputMethod</key>
                    <dict/>
                    <key>shell</key>
                    <dict/>
                    <key>source</key>
                    <dict/>
                </dict>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key>
                <string>Run Shell Script</string>
                <key>ActionParameters</key>
                <dict>
                    <key>COMMANDString</key>
                    <string>{script_content}</string>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
                <key>BundleIdentifier</key>
                <string>com.apple.RunShellScript</string>
                <key>CFBundleVersion</key>
                <string>2.0.3</string>
                <key>CanShowSelectedItemsWhenRun</key>
                <false/>
                <key>CanShowWhenRun</key>
                <true/>
                <key>Category</key>
                <array>
                    <string>AMCategoryUtilities</string>
                </array>
                <key>Class Name</key>
                <string>RunShellScriptAction</string>
                <key>InputUUID</key>
                <string>12345678-1234-1234-1234-123456789012</string>
                <key>Keywords</key>
                <array>
                    <string>Shell</string>
                    <string>Script</string>
                    <string>Command</string>
                    <string>Run</string>
                    <string>Unix</string>
                </array>
                <key>OutputUUID</key>
                <string>12345678-1234-1234-1234-123456789013</string>
                <key>UUID</key>
                <string>12345678-1234-1234-1234-123456789014</string>
                <key>UnlocalizedApplications</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>arguments</key>
                <dict>
                    <key>0</key>
                    <dict>
                        <key>default value</key>
                        <integer>0</integer>
                        <key>name</key>
                        <string>inputMethod</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>0</string>
                    </dict>
                    <key>1</key>
                    <dict>
                        <key>default value</key>
                        <false/>
                        <key>name</key>
                        <string>CheckedForUserDefaultShell</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>1</string>
                    </dict>
                    <key>2</key>
                    <dict>
                        <key>default value</key>
                        <string></string>
                        <key>name</key>
                        <string>source</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>2</string>
                    </dict>
                    <key>3</key>
                    <dict>
                        <key>default value</key>
                        <string></string>
                        <key>name</key>
                        <string>COMMANDString</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>3</string>
                    </dict>
                    <key>4</key>
                    <dict>
                        <key>default value</key>
                        <string>/bin/sh</string>
                        <key>name</key>
                        <string>shell</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>4</string>
                    </dict>
                </dict>
                <key>isViewVisible</key>
                <integer>1</integer>
                <key>location</key>
                <string>309.000000:252.000000</string>
                <key>nibPath</key>
                <string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib</string>
            </dict>
            <key>isViewVisible</key>
            <integer>1</integer>
        </dict>
    </array>
    <key>connectors</key>
    <dict/>
    <key>workflowMetaData</key>
    <dict>
        <key>applicationBundleIDsByPath</key>
        <dict/>
        <key>applicationPaths</key>
        <array/>
        <key>inputTypeIdentifier</key>
        <string>com.apple.Automator.fileSystemObject</string>
        <key>outputTypeIdentifier</key>
        <string>com.apple.Automator.nothing</string>
        <key>presentationMode</key>
        <integer>15</integer>
        <key>processesInput</key>
        <false/>
        <key>serviceInputTypeIdentifier</key>
        <string>com.apple.Automator.fileSystemObject</string>
        <key>serviceOutputTypeIdentifier</key>
        <string>com.apple.Automator.nothing</string>
        <key>serviceProcessesInput</key>
        <false/>
        <key>systemImageName</key>
        <string>NSTouchBarPlay</string>
        <key>useAutomaticInputType</key>
        <false/>
        <key>workflowTypeIdentifier</key>
        <string>com.apple.Automator.servicesMenu</string>
    </dict>
</dict>
</plist>"""
        with open(os.path.join(workflow_dir, "Contents", "Info.plist"), "w") as f:
            f.write(info_plist)
        with open(os.path.join(workflow_dir, "Contents", "document.wflow"), "w") as f:
            f.write(document_wflow)

        return True, "Успешно! Пункт добавлен в Quick Actions / Services (Finder)."

    def remove(self) -> Tuple[bool, str]:
        import os, shutil
        workflow_dir = os.path.expanduser("~/Library/Services/CodeContextAI.workflow")
        if os.path.exists(workflow_dir):
            shutil.rmtree(workflow_dir)
            return True, "Успешно! Пункт удален из Quick Actions."
        return False, "Интеграция не найдена."