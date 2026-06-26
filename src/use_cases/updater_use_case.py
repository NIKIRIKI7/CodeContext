import os
import sys
from ..services.updater_service import check_for_updates as updater_check, download_and_install
from ..store.state import AppState
from ..utils.logger import app_logger
from src.i18n import tr


async def check_for_updates(state: AppState, current_version: str):
    state.add_log(tr("updater_use_case.checking", version=current_version))
    app_logger.info(f"UI Trigger: Start update check. Version: {current_version}")

    state.update_info = {
        'status': 'checking',
        'version': current_version,
        'notes': tr("updater_use_case.connecting")
    }
    state.show_update = True
    state.notify()

    try:
        update_info = await updater_check(
            current_version,
            state.settings.receive_prereleases
        )

        if update_info:
            state.add_log(tr("updater_use_case.update_found", version=update_info['version']))
            update_info['status'] = 'available'
        else:
            state.add_log(tr("updater_use_case.up_to_date"))
            update_info = {
                'status': 'latest',
                'version': current_version,
                'notes': tr("updater_use_case.no_updates")
            }

        state.update_info = update_info
        state.notify()

    except Exception as e:
        err_msg = str(e)
        state.add_log(tr("updater_use_case.check_error", error=err_msg))
        state.update_info = {
            'status': 'error',
            'version': current_version,
            'notes': tr("updater_use_case.check_error_notes", error=err_msg)
        }
        state.notify()


async def apply_update(state: AppState, download_url: str):
    state.add_log(tr("updater_use_case.download_starting"))
    app_logger.info("UI Trigger: User accepted update download.")

    def progress_callback(progress: float):
        state.status_message = tr("updater_use_case.download_progress", percent=int(progress * 100))
        state.progress = progress
        state.update_info = {**state.update_info, 'status': 'downloading', 'progress': progress}
        state.notify()

    try:
        state.update_info = {**state.update_info, 'status': 'downloading', 'progress': 0.0}
        state.notify()

        success = await download_and_install(download_url, progress_callback)
        if success:
            state.add_log(tr("updater_use_case.update_installed"))
            state.status_message = tr("updater_use_case.restarting")
            state.progress = 1.0
            state.notify()
            app_logger.info("Restarting application to apply update...")

            os.execv(sys.executable, [sys.executable] + sys.argv)
    except Exception as e:
        err_msg = str(e)
        state.add_log(tr("updater_use_case.install_error", error=err_msg))
        state.update_info = {
            'status': 'error',
            'version': 'update_failed',
            'notes': tr("updater_use_case.install_error_notes", error=err_msg)
        }
        state.notify()
