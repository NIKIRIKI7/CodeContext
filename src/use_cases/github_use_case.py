"""
GitHubUseCase — сценарий клонирования GitHub-репозитория.

Ответственность: запустить клонирование, добавить путь в Store,
сообщить о прогрессе. Не знает о GitHubService внутренностях.
"""

from ..actions.action_types import (
    UI_SET_LOADING, UI_UPDATE_STATUS, UI_ADD_LOG,
    GITHUB_CLONE_SUCCESS, GITHUB_CLONE_FAILURE,
)
from ..actions.dispatcher import Dispatcher
from ..services.github_service import GitHubService


class GitHubUseCase:
    """Клонирует репозиторий и публикует результат в Store."""

    def __init__(self, dispatcher: Dispatcher, github_service: GitHubService):
        self._dispatcher = dispatcher
        self._github_service = github_service

    async def execute(self, url: str) -> None:
        """
        Клонирует репозиторий по URL.
        Результат (путь к temp-папке) публикуется через GITHUB_CLONE_SUCCESS.
        """
        if not url:
            return

        self._dispatcher.dispatch(UI_SET_LOADING, True)
        self._dispatcher.dispatch(UI_UPDATE_STATUS, {
            'message': "Клонирование...", 'progress': 0.0
        })
        self._dispatcher.dispatch(UI_ADD_LOG, f"GitHub: клонирование {url}")

        try:
            temp_path = await self._github_service.clone_repo_async(url)
            self._dispatcher.dispatch(GITHUB_CLONE_SUCCESS, temp_path)
            self._dispatcher.dispatch(UI_ADD_LOG, f"✅ Репозиторий загружен: {temp_path}")

        except Exception as exc:
            self._dispatcher.dispatch(GITHUB_CLONE_FAILURE, str(exc))
            self._dispatcher.dispatch(UI_ADD_LOG, f"❌ GitHub Error: {exc}")

        finally:
            self._dispatcher.dispatch(UI_SET_LOADING, False)
            self._dispatcher.dispatch(UI_UPDATE_STATUS, {
                'message': "Готово", 'progress': 0.0
            })