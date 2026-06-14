import argparse
import os
import sys
import platform

sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from src.di_container import DIContainer
from src.utils.async_runtime import AsyncRuntime
from src.utils.logger import app_logger
from src.i18n import tr
from PySide6.QtWidgets import QApplication
from PySide6.QtGui import QIcon, QImage, QPainter, QColor, QFont, QPen
from PySide6.QtCore import Qt
from PySide6.QtNetwork import QLocalServer, QLocalSocket
from src.ui.theme_manager import ThemeManager
from src.ui.main_window import MainWindow


def get_resource_path(relative_path: str) -> str:
    """Поиск ресурса с подъёмом вверх от main.py (Themes, Assets, VERSION.txt)"""
    if getattr(sys, 'frozen', False):
        return os.path.join(sys._MEIPASS, relative_path)

    start = os.path.dirname(os.path.abspath(__file__))
    d = start
    while True:
        candidate = os.path.join(d, relative_path)
        if os.path.exists(candidate):
            return candidate
        parent = os.path.dirname(d)
        if parent == d:
            break
        d = parent
    return os.path.join(start, relative_path)


def create_default_logo(path: str):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img = QImage(256, 256, QImage.Format_ARGB32)
    img.fill(Qt.transparent)
    painter = QPainter(img)
    painter.setRenderHint(QPainter.Antialiasing)
    painter.setBrush(QColor("#0071e3"))
    painter.setPen(Qt.NoPen)
    painter.drawRoundedRect(12, 12, 232, 232, 48, 48)
    painter.setBrush(QColor("#0a84ff"))
    painter.drawRoundedRect(24, 24, 208, 208, 36, 36)
    painter.setPen(QPen(Qt.white, 16))
    font = QFont("Consolas", 64, QFont.Bold)
    painter.setFont(font)
    painter.drawText(img.rect(), Qt.AlignCenter, "{ C }")
    painter.end()
    img.save(path, "PNG")

def send_to_existing_instance(path: str) -> bool:
    socket = QLocalSocket()
    socket.connectToServer("CodeContextAI_IPC")
    if socket.waitForConnected(500):
        socket.write(path.encode('utf-8'))
        socket.waitForBytesWritten(500)
        socket.disconnectFromServer()
        return True
    return False


def main():
    app_logger.info("=" * 50)
    app_logger.info("🚀 CodeContext AI Started")

    if getattr(sys, 'frozen', False):
        exe_path = sys.executable
        if platform.system() == "Darwin":
            parts = exe_path.split(os.sep)
            if "Contents" in parts and "MacOS" in parts:
                app_idx = parts.index("Contents") - 1
                app_path = os.sep.join(parts[:app_idx+1])
                old_app = app_path + ".old"
                if os.path.exists(old_app):
                    import shutil
                    try: shutil.rmtree(old_app, ignore_errors=True)
                    except Exception: pass
        else:
            old_path = exe_path + ".old"
            if os.path.exists(old_path):
                try: os.remove(old_path)
                except Exception: pass

    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", action="store_true", help=tr("main.cli_mode"))
    parser.add_argument("--path", type=str, help=tr("main.path"))
    parser.add_argument("--patch", type=str, help=tr("main.patch"))
    parser.add_argument("--mode", type=str, default="default", help=tr("main.mode"))
    parser.add_argument("--install-context", action="store_true", help=tr("main.install_context"))
    parser.add_argument("--remove-context", action="store_true", help=tr("main.remove_context"))
    parser.add_argument("--minify", type=str, choices=["true", "false"], help=tr("main.minify"))
    parser.add_argument("--skeleton", type=str, choices=["true", "false"], help=tr("main.skeleton"))
    parser.add_argument("--format", type=str, choices=["markdown", "xml", "plain", "jsonl_chunk"], help=tr("main.format"))
    parser.add_argument("--dry-run", action="store_true", help=tr("main.dry_run"))
    parser.add_argument("--silent", action="store_true", help=tr("main.silent"))
    parser.add_argument("--stdout", action="store_true", help=tr("main.stdout"))
    parser.add_argument("--fail-if-exceeds", type=int, help=tr("main.fail_if_exceeds"))
    parser.add_argument("--git", action="store_true", help="Scan only git modified files")
    parser.add_argument("--git-base", type=str, default="", help="Base branch for git diff in CI/CD (e.g., origin/main)")

    args, _ = parser.parse_known_args()
    container = DIContainer()

    if args.install_context:
        container.integration_service.install_context_menu()
        sys.exit(0)

    if args.remove_context:
        container.integration_service.remove_context_menu()
        sys.exit(0)

    if args.cli or args.silent or args.dry_run or args.stdout:
        if not args.path:
            args.path = os.getcwd()
        kwargs = {
            'mode': args.mode,
            'minify': args.minify,
            'skeleton': args.skeleton,
            'format': args.format,
            'dry_run': args.dry_run,
            'silent': args.silent or args.stdout,
            'stdout': args.stdout,
            'fail_if_exceeds': args.fail_if_exceeds,
            'git': args.git,
            'git_base': args.git_base,
        }
        if args.patch:
            container.cli_controller.run_patch(args.path, args.patch)
        else:
            container.cli_controller.run(args.path, **kwargs)
        sys.exit(0)

    if args.path and os.path.exists(args.path):
        abs_path = os.path.abspath(args.path)
        if send_to_existing_instance(abs_path):
            app_logger.info(tr("main.sent_to_existing"))
            sys.exit(0)

    app_logger.info("Starting GUI Mode (PySide6)...")

    os.environ["QT_AUTO_SCREEN_SCALE_FACTOR"] = "1"
    os.environ["QT_ENABLE_HIGHDPI_SCALING"] = "1"

    AsyncRuntime.start()
    app = QApplication(sys.argv)

    logo_path = get_resource_path(os.path.join("assets", "images", "logo.png"))
    if not os.path.exists(logo_path):
        try:
            create_default_logo(logo_path)
        except Exception as e:
            app_logger.error(f"Failed to generate logo: {e}")

    if os.path.exists(logo_path):
        app.setWindowIcon(QIcon(logo_path))

    built_in_themes = get_resource_path("themes")
    from src.utils.config import get_app_data_dir
    user_themes_dir = os.path.join(get_app_data_dir(), "themes")
    os.makedirs(user_themes_dir, exist_ok=True)

    ThemeManager.load_themes(built_in_themes, user_themes_dir)

    if "apple" in ThemeManager.get_available_themes():
        ThemeManager.apply_theme("apple", "light")
    elif ThemeManager.get_available_themes():
        ThemeManager.apply_theme(ThemeManager.get_available_themes()[0], "light")

    window = MainWindow(container.store, container.main_controller)
    if os.path.exists(logo_path):
        window.setWindowIcon(QIcon(logo_path))

    ipc_server = QLocalServer()
    ipc_server.removeServer("CodeContextAI_IPC")
    ipc_server.listen("CodeContextAI_IPC")

    def on_new_connection():
        socket = ipc_server.nextPendingConnection()
        if socket.waitForReadyRead(500):
            received_path = socket.readAll().data().decode('utf-8')
            if received_path and os.path.exists(received_path):
                app_logger.info(f"IPC Received: {received_path}")
                container.main_controller.add_folder(received_path)
                window.activateWindow()
                window.raise_()

    ipc_server.newConnection.connect(on_new_connection)

    if args.path and os.path.exists(args.path):
        container.main_controller.add_folder(os.path.abspath(args.path))

    window.show()
    exit_code = app.exec()

    ipc_server.close()
    AsyncRuntime.stop()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
