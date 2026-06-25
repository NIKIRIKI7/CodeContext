import os
import sys
from ..services.updater_service import UpdaterService
from ..store.state import AppState
from ..utils.logger import app_logger
from src.i18n import tr


class UpdaterUseCase:
    def __init__(self, state: AppState, updater_service: UpdaterService):
        self.state = state
        self._updater_service = updater_service

    async def check_for_updates(self, state: AppState, current_version: str):
        self.state.add_log(tr("updater_use_case.checking", version=current_version))
        app_logger.info(f"UI Trigger: Start update check. Version: {current_version}")

        self.state.update_info = {
            'status': 'checking',
            'version': current_version,
            'notes': tr("updater_use_case.connecting")
        }
        self.state.show_update = True
        self.state.notify()

        try:
            update_info = await self._updater_service.check_for_updates(
                current_version,
                state.settings.receive_prereleases
            )

            if update_info:
                self.state.add_log(tr("updater_use_case.update_found", version=update_info['version']))
                update_info['status'] = 'available'
            else:
                self.state.add_log(tr("updater_use_case.up_to_date"))
                update_info = {
                    'status': 'latest',
                    'version': current_version,
                    'notes': tr("updater_use_case.no_updates")
                }

            self.state.update_info = update_info
            self.state.notify()

        except Exception as e:
            err_msg = str(e)
            self.state.add_log(tr("updater_use_case.check_error", error=err_msg))
            self.state.update_info = {
                'status': 'error',
                'version': current_version,
                'notes': tr("updater_use_case.check_error_notes", error=err_msg)
            }
            self.state.notify()

    async def apply_update(self, download_url: str):
        self.state.add_log(tr("updater_use_case.download_starting"))
        app_logger.info("UI Trigger: User accepted update download.")

        def progress_callback(progress: float):
            self.state.status_message = tr("updater_use_case.download_progress", percent=int(progress * 100))
            self.state.progress = progress
            self.state.update_info = {**self.state.update_info, 'status': 'downloading', 'progress': progress}
            self.state.notify()

        try:
            self.state.update_info = {**self.state.update_info, 'status': 'downloading', 'progress': 0.0}
            self.state.notify()

            success = await self._updater_service.download_and_install(download_url, progress_callback)
            if success:
                self.state.add_log(tr("updater_use_case.update_installed"))
                self.state.status_message = tr("updater_use_case.restarting")
                self.state.progress = 1.0
                self.state.notify()
                app_logger.info("Restarting application to apply update...")

                os.execv(sys.executable, [sys.executable] + sys.argv)
        except Exception as e:
            err_msg = str(e)
            self.state.add_log(tr("updater_use_case.install_error", error=err_msg))
            self.state.update_info = {
                'status': 'error',
                'version': 'update_failed',
                'notes': tr("updater_use_case.install_error_notes", error=err_msg)
            }
            self.state.notify()
