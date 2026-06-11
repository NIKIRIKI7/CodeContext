import sys
import subprocess
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import UI_UPDATE_STATUS, UI_ADD_LOG, UI_SHOW_UPDATE
from ..services.updater_service import UpdaterService
from ..store.state import AppState
from ..utils.logger import app_logger
from src.i18n import tr


class UpdaterUseCase:
    """Оркестрирует проверку и установку обновлений, взаимодействуя с Redux Store."""

    def __init__(self, dispatcher: Dispatcher, updater_service: UpdaterService):
        self._dispatcher = dispatcher
        self._updater_service = updater_service

    async def check_for_updates(self, state: AppState, current_version: str):
        self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.checking", version=current_version))
        app_logger.info(f"UI Trigger: Start update check. Version: {current_version}")

        self._dispatcher.dispatch(UI_SHOW_UPDATE, {
            'status': 'checking',
            'version': current_version,
            'notes': tr("updater_use_case.connecting")
        })

        try:
            update_info = await self._updater_service.check_for_updates(
                current_version,
                state.settings.receive_prereleases
            )

            if update_info:
                self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.update_found", version=update_info['version']))
                update_info['status'] = 'available'
                self._dispatcher.dispatch(UI_SHOW_UPDATE, update_info)
            else:
                self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.up_to_date"))
                self._dispatcher.dispatch(UI_SHOW_UPDATE, {
                    'status': 'latest',
                    'version': current_version,
                    'notes': tr("updater_use_case.no_updates")
                })
        except Exception as e:
            err_msg = str(e)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.check_error", error=err_msg))
            self._dispatcher.dispatch(UI_SHOW_UPDATE, {
                'status': 'error',
                'version': current_version,
                'notes': tr("updater_use_case.check_error_notes", error=err_msg)
            })

    async def apply_update(self, download_url: str):
        self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.download_starting"))
        app_logger.info("UI Trigger: User accepted update download.")

        def progress_callback(progress: float):
            self._dispatcher.dispatch(UI_UPDATE_STATUS, {
                'message': tr("updater_use_case.download_progress", percent=int(progress * 100)),
                'progress': progress
            })

        try:
            success = await self._updater_service.download_and_install(download_url, progress_callback)
            if success:
                self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.update_installed"))
                self._dispatcher.dispatch(UI_UPDATE_STATUS, {'message': tr("updater_use_case.restarting"), 'progress': 1.0})
                app_logger.info("Restarting application to apply update...")
                subprocess.Popen([sys.executable])
                sys.exit(0)
        except Exception as e:
            err_msg = str(e)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("updater_use_case.install_error", error=err_msg))
            self._dispatcher.dispatch(UI_SHOW_UPDATE, {
                'status': 'error',
                'version': 'update_failed',
                'notes': tr("updater_use_case.install_error_notes", error=err_msg)
            })
