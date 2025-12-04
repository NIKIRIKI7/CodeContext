"""
Script to register Windows Context Menu entry for CodeContext AI.
Run once.
"""
import sys
import os
import winreg
import ctypes

def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def register_context_menu():
    # Paths
    python_exe = sys.executable
    # Assuming main.py is in the same folder as this script
    script_path = os.path.join(os.getcwd(), "main.py")
    
    if not os.path.exists(script_path):
        print(f"Error: Could not find {script_path}")
        return

    # Icon path (optional, using python icon for now)
    icon_path = python_exe

    # Command: python "path/to/main.py" --cli --path "%1"
    command = f'"{python_exe}" "{script_path}" --cli --path "%1"'
    
    key_path = r"Directory\shell\CodeContextAI"
    
    try:
        # Create Key
        key = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, key_path)
        winreg.SetValue(key, "", winreg.REG_SZ, "Scan with CodeContext AI")
        winreg.SetValueEx(key, "Icon", 0, winreg.REG_SZ, icon_path)
        
        # Create Command Subkey
        command_key = winreg.CreateKey(key, "command")
        winreg.SetValue(command_key, "", winreg.REG_SZ, command)
        
        winreg.CloseKey(command_key)
        winreg.CloseKey(key)
        
        print("✅ Успешно! Теперь вы можете нажать ПКМ на папку -> 'Scan with CodeContext AI'")
        
    except Exception as e:
        print(f"❌ Ошибка при записи в реестр: {e}")

if __name__ == "__main__":
    if is_admin():
        register_context_menu()
        input("Нажмите Enter для выхода...")
    else:
        # Re-run as admin
        ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, " ".join(sys.argv), None, 1)