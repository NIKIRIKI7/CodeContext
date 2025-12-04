"""
Script to REMOVE the Windows Context Menu entry for CodeContext AI.
Run as Administrator.
"""
import sys
import ctypes
import winreg


def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False


def remove_context_menu():
    key_path = r"Directory\shell\CodeContextAI"

    try:
        # We need to delete the subkey "command" first, then the parent key
        # winreg.DeleteKey only deletes empty keys.

        # 1. Delete 'command' subkey
        try:
            winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, key_path + r"\command")
        except FileNotFoundError:
            pass  # Already gone

        # 2. Delete main key
        winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, key_path)

        print("✅ Успешно! Пункт меню удален.")
        ctypes.windll.user32.MessageBoxW(0, "Пункт меню 'Scan with CodeContext AI' успешно удален.", "Успех", 0x40)

    except FileNotFoundError:
        print("ℹ️ Ключ не найден (возможно, уже удален).")
    except PermissionError:
        print("❌ Ошибка прав доступа. Запустите от имени Администратора.")
    except Exception as e:
        print(f"❌ Ошибка: {e}")


if __name__ == "__main__":
    if is_admin():
        remove_context_menu()
        input("Нажмите Enter для выхода...")
    else:
        # Re-run as admin
        ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, " ".join(sys.argv), None, 1)