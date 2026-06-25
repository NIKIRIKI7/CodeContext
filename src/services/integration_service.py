import os
import sys
import uuid
import shutil
import platform
import plistlib
import ctypes
from typing import Tuple, Optional
from src.i18n import tr

class IntegrationService:
    def install_context_menu(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        sys_os = platform.system()
        try:
            if sys_os == "Windows":
                return self._install_win(custom_python_path)
            elif sys_os == "Linux":
                return self._install_linux(custom_python_path)
            elif sys_os == "Darwin":
                return self._install_mac(custom_python_path)
            return False, "Unsupported OS"
        except Exception as e:
            return False, f"Error: {e}"

    def remove_context_menu(self) -> Tuple[bool, str]:
        sys_os = platform.system()
        try:
            if sys_os == "Windows":
                return self._remove_win()
            elif sys_os == "Linux":
                return self._remove_linux()
            elif sys_os == "Darwin":
                return self._remove_mac()
            return False, "Unsupported OS"
        except Exception as e:
            return False, f"Error: {e}"

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        try:
            if platform.system() == "Windows":
                return self._install_cli_win(custom_python_path)
            else:
                return self._install_cli_unix(custom_python_path)
        except Exception as e:
            return False, f"Error: {e}"

    def remove_cli(self) -> Tuple[bool, str]:
        try:
            if platform.system() == "Windows":
                return self._remove_cli_win()
            else:
                return self._remove_cli_unix()
        except Exception as e:
            return False, f"Error: {e}"

    # --- Windows ---
    def _install_win(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        import winreg
        if getattr(sys, 'frozen', False):
            exe_path = sys.executable
            command_base = f'"{exe_path}" --cli --path'
            gui_command = f'"{exe_path}" --path'
            icon_path = exe_path
        else:
            python_exe = custom_python_path if custom_python_path and os.path.exists(custom_python_path) else sys.executable
            script_path = os.path.abspath(sys.argv[0])
            if not script_path.endswith("main.py"):
                if os.path.exists(os.path.join(os.getcwd(), "main.py")):
                    script_path = os.path.join(os.getcwd(), "main.py")
            command_base = f'"{python_exe}" "{script_path}" --cli --path'
            gui_command = f'"{python_exe}" "{script_path}" --path'
            icon_path = python_exe

        def _add_menu(root_key, path, target_arg):
            key = winreg.CreateKey(root_key, path)
            try: winreg.DeleteValue(key, "")
            except OSError: pass
            winreg.SetValueEx(key, "MUIVerb", 0, winreg.REG_SZ, "CodeContext AI")
            winreg.SetValueEx(key, "Icon", 0, winreg.REG_SZ, icon_path)
            winreg.SetValueEx(key, "SubCommands", 0, winreg.REG_SZ, "")
            
            sub_shell = winreg.CreateKey(key, "shell")
            sub_gui = winreg.CreateKey(sub_shell, "cmd_gui")
            winreg.SetValue(sub_gui, "", winreg.REG_SZ, tr("integration_strategies.windows.menu_open_ui"))
            winreg.SetValueEx(sub_gui, "Icon", 0, winreg.REG_SZ, icon_path)
            cmd_gui = winreg.CreateKey(sub_gui, "command")
            winreg.SetValue(cmd_gui, "", winreg.REG_SZ, f'{gui_command} "{target_arg}"')
            winreg.CloseKey(cmd_gui)
            winreg.CloseKey(sub_gui)
            
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

        _add_menu(winreg.HKEY_CURRENT_USER, r"Software\Classes\Directory\shell\CodeContextAI", "%1")
        _add_menu(winreg.HKEY_CURRENT_USER, r"Software\Classes\Directory\Background\shell\CodeContextAI", "%V")
        _add_menu(winreg.HKEY_CURRENT_USER, r"Software\Classes\*\shell\CodeContextAI", "%1")
        return True, tr("integration_strategies.windows.install_success")

    def _delete_registry_tree(self, root_key, key_path: str):
        import winreg
        try:
            open_key = winreg.OpenKey(root_key, key_path, 0, winreg.KEY_ALL_ACCESS)
            while winreg.QueryInfoKey(open_key)[0] > 0:
                self._delete_registry_tree(open_key, winreg.EnumKey(open_key, 0))
            winreg.CloseKey(open_key)
            winreg.DeleteKey(root_key, key_path)
        except FileNotFoundError: pass

    def _remove_win(self) -> Tuple[bool, str]:
        import winreg
        for path in (r"Software\Classes\Directory\shell\CodeContextAI", r"Software\Classes\Directory\Background\shell\CodeContextAI", r"Software\Classes\*\shell\CodeContextAI"):
            self._delete_registry_tree(winreg.HKEY_CURRENT_USER, path)
        return True, tr("integration_strategies.windows.remove_success")

    def _install_cli_win(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        import winreg
        from ..utils.config import get_app_data_dir
        is_frozen = getattr(sys, 'frozen', False)
        exe_path = sys.executable
        python_exe = custom_python_path if custom_python_path and os.path.exists(custom_python_path) else sys.executable
        script_path = os.path.abspath(sys.argv[0])
        bin_dir = os.path.join(get_app_data_dir(), "bin")
        
        buf = ctypes.create_unicode_buffer(260)
        ctypes.windll.kernel32.GetShortPathNameW(bin_dir, buf, 260)
        short_bin = buf.value or bin_dir
        os.makedirs(short_bin, exist_ok=True)
        
        with open(os.path.join(short_bin, "codecontext.bat"), "w", encoding="utf-8") as f:
            f.write('@echo off\nchcp 65001 > nul\nset PYTHONIOENCODING=utf-8\n')
            if is_frozen: f.write(f'"{exe_path}" %*\n')
            else: f.write(f'"{python_exe}" "{script_path}" %*\n')
            
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_ALL_ACCESS)
        path_val = ""
        try: path_val, _ = winreg.QueryValueEx(key, "Path")
        except FileNotFoundError: pass
        
        if bin_dir not in path_val.split(os.pathsep):
            new_path = path_val + (os.pathsep if path_val and not path_val.endswith(os.pathsep) else "") + bin_dir
            winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, new_path)
            ctypes.windll.user32.SendMessageTimeoutW(0xFFFF, 0x001A, 0, "Environment", 0x0002, 5000, None)
        winreg.CloseKey(key)
        return True, tr("integration_strategies.windows.cli_install_success")

    def _remove_cli_win(self) -> Tuple[bool, str]:
        import winreg
        from ..utils.config import get_app_data_dir
        bin_dir = os.path.join(get_app_data_dir(), "bin")
        shutil.rmtree(bin_dir, ignore_errors=True)
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_ALL_ACCESS)
        try:
            path_val, _ = winreg.QueryValueEx(key, "Path")
            paths = [p for p in path_val.split(os.pathsep) if p and p != bin_dir]
            winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, os.pathsep.join(paths))
            ctypes.windll.user32.SendMessageTimeoutW(0xFFFF, 0x001A, 0, "Environment", 0x0002, 5000, None)
        except FileNotFoundError: pass
        finally: winreg.CloseKey(key)
        return True, tr("integration_strategies.windows.cli_remove_success")

    # --- Unix / Mac ---
    def _install_linux(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        python_exe = custom_python_path or sys.executable
        script_path = os.path.abspath(sys.argv[0])
        kde_dir = os.path.expanduser("~/.local/share/kio/servicemenus")
        os.makedirs(kde_dir, exist_ok=True)
        with open(os.path.join(kde_dir, "codecontext_ai.desktop"), "w") as f:
            f.write(f"[Desktop Entry]\nType=Service\nServiceTypes=KonqPopupMenu/Plugin\n"
                    f"MimeType=all/allfiles;inode/directory;\nActions=OpenGUI;CopyXML;CopySkeleton;\n"
                    f"X-KDE-Priority=TopLevel\nX-KDE-Submenu=CodeContext AI\nIcon=utilities-terminal\n\n"
                    f"[Desktop Action OpenGUI]\nName={tr('integration_strategies.linux.menu_open_ui')}\n"
                    f"Icon=utilities-terminal\nExec={python_exe} {script_path} --path %f\n\n"
                    f"[Desktop Action CopyXML]\nName={tr('integration_strategies.linux.menu_copy_xml')}\n"
                    f"Icon=edit-copy\nExec={python_exe} {script_path} --cli --path %f --format xml --silent\n\n"
                    f"[Desktop Action CopySkeleton]\nName={tr('integration_strategies.linux.menu_copy_skeleton')}\n"
                    f"Icon=edit-copy\nExec={python_exe} {script_path} --cli --path %f --skeleton true --silent\n")
        return True, tr("integration_strategies.linux.install_success")

    def _remove_linux(self) -> Tuple[bool, str]:
        for path in ("~/.local/share/file-manager/actions/codecontext_ai.desktop", "~/.local/share/kio/servicemenus/codecontext_ai.desktop"):
            if os.path.exists(os.path.expanduser(path)): os.remove(os.path.expanduser(path))
        return True, tr("integration_strategies.linux.remove_success")

    def _install_cli_unix(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        bin_dir = os.path.expanduser("~/.local/bin")
        os.makedirs(bin_dir, exist_ok=True)
        sh_path = os.path.join(bin_dir, "codecontext")
        with open(sh_path, "w", encoding="utf-8") as f:
            if getattr(sys, 'frozen', False): f.write(f'#!/bin/bash\nexec "{sys.executable}" "$@"\n')
            else: f.write(f'#!/bin/bash\nexec "{custom_python_path or sys.executable}" "{os.path.abspath(sys.argv[0])}" "$@"\n')
        os.chmod(sh_path, 0o755)
        return True, tr("integration_strategies.linux.cli_install_success")

    def _remove_cli_unix(self) -> Tuple[bool, str]:
        sh_path = os.path.expanduser("~/.local/bin/codecontext")
        if os.path.exists(sh_path): os.remove(sh_path)
        return True, tr("integration_strategies.linux.cli_remove_success")

    def _install_mac(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        # ponytail: Mac Automator actions integration (simplified from original)
        exe = sys.executable if getattr(sys, 'frozen', False) else f"{custom_python_path or sys.executable} {os.path.abspath(sys.argv[0])}"
        cmds = {
            "CodeContext AI": f'"{exe}" --path "$1" > /dev/null 2>&1 &',
            "CodeContext AI (XML)": f'"{exe}" --cli --path "$1" --format xml --silent > /dev/null 2>&1 &',
            "CodeContext AI (Skeleton)": f'"{exe}" --cli --path "$1" --skeleton true --silent > /dev/null 2>&1 &',
        }
        services_dir = os.path.expanduser("~/Library/Services")
        os.makedirs(services_dir, exist_ok=True)
        for name, command in cmds.items():
            workflow_path = os.path.join(services_dir, f"{name}.workflow")
            shutil.rmtree(workflow_path, ignore_errors=True)
            os.makedirs(os.path.join(workflow_path, "Contents"), exist_ok=True)
            uid = lambda: str(uuid.uuid4()).upper()
            plistlib.dump({
                "CFBundleIdentifier": f"com.codecontext.ai.{uuid.uuid4().hex}", "CFBundleName": name,
                "CFBundlePackageType": "BNDL", "CFBundleShortVersionString": "1.0", "CFBundleVersion": "1.0",
                "NSServices": [{"NSMenuItem": {"default": name}, "NSMessage": "runWorkflowAsService", "NSSendFileTypes": ["public.folder", "public.directory"]}],
            }, open(os.path.join(workflow_path, "Contents", "Info.plist"), "wb"))
            plistlib.dump({
                "AMApplicationBuild": "523", "AMApplicationVersion": "2.10", "AMDocumentVersion": "2",
                "actions": [{"action": {"ActionName": "Run Shell Script", "ActionParameters": {"COMMANDString": command, "CheckedForUserDefaultShell": True, "inputMethod": 1, "shell": "/bin/bash", "source": ""}, "Class Name": "RunShellScriptAction"}}],
            }, open(os.path.join(workflow_path, "Contents", "document.wflow"), "wb"))
        import subprocess; subprocess.run(["/System/Library/CoreServices/pbs", "-flush"], check=False)
        return True, tr("integration_strategies.macos.install_success")

    def _remove_mac(self) -> Tuple[bool, str]:
        services_dir = os.path.expanduser("~/Library/Services")
        for name in ["CodeContext AI", "CodeContext AI (XML)", "CodeContext AI (Skeleton)"]:
            shutil.rmtree(os.path.join(services_dir, f"{name}.workflow"), ignore_errors=True)
        import subprocess; subprocess.run(["/System/Library/CoreServices/pbs", "-flush"], check=False)
        return True, tr("integration_strategies.macos.remove_success")
