import os
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

    async def execute(self, url: str, base_dest_path: str = "") -> None:
        """
        Клонирует репозиторий по URL.
        Если base_dest_path пустой - использует временную папку.
        """
        if not url:
            return

        self._dispatcher.dispatch(UI_SET_LOADING, True)
        self._dispatcher.dispatch(UI_UPDATE_STATUS, {
            'message': "Клонирование...", 'progress': 0.0
        })
        self._dispatcher.dispatch(UI_ADD_LOG, f"GitHub: клонирование {url}")

        try:
            target_path = None
            is_temp = True

            if base_dest_path:
                # Извлекаем имя репозитория из URL (например, 'CodeContext' из '.../CodeContext.git')
                repo_name = url.rstrip('/').split('/')[-1]
                if repo_name.endswith('.git'):
                    repo_name = repo_name[:-4]
                if not repo_name:
                    repo_name = "github_repo"

                target_path = os.path.join(base_dest_path, repo_name)
                is_temp = False

            final_path = await self._github_service.clone_repo_async(url, target_path)

            # Передаем словарь с флагом is_temp, чтобы Store знал, удалять ли папку при закрытии программы
            self._dispatcher.dispatch(GITHUB_CLONE_SUCCESS, {
                "path": final_path,
                "is_temp": is_temp
            })

            self._dispatcher.dispatch(UI_ADD_LOG, f"✅ Репозиторий загружен: {final_path}")

        except Exception as exc:
            self._dispatcher.dispatch(GITHUB_CLONE_FAILURE, str(exc))
            self._dispatcher.dispatch(UI_ADD_LOG, f"❌ GitHub Error: {exc}")
        finally:
            self._dispatcher.dispatch(UI_SET_LOADING, False)
            self._dispatcher.dispatch(UI_UPDATE_STATUS, {
                'message': "Готово", 'progress': 0.0
            })