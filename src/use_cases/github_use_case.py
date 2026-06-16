import os
import re
from ..services.github_service import GitHubService
from ..store.state import AppState
from src.i18n import tr


class GitHubUseCase:
    def __init__(self, state: AppState, github_service: GitHubService):
        self.state = state
        self._github_service = github_service

    async def execute(self, url: str, base_dest_path: str = "") -> None:
        if not url:
            return

        self.state.is_loading = True
        self.state.status_message = tr("github_use_case.preparing")
        self.state.progress = 0.0
        self.state.notify()

        pr_files = []
        is_pr = "/pull/" in url
        clone_url = url

        try:
            if is_pr:
                self.state.add_log(tr("github_use_case.pr_detected"))
                pr_files = await self._github_service.fetch_pr_files_async(url)
                self.state.add_log(tr("github_use_case.pr_files_count", count=len(pr_files)))
                clone_url = re.sub(r'/pull/\d+.*$', '', url)

            self.state.status_message = tr("github_use_case.cloning")
            self.state.progress = 0.3
            self.state.notify()
            self.state.add_log(tr("github_use_case.cloning_url", url=clone_url))

            target_path = None
            if base_dest_path:
                repo_name = clone_url.rstrip('/').split('/')[-1]
                if repo_name.endswith('.git'):
                    repo_name = repo_name[:-4]
                if not repo_name:
                    repo_name = "github_repo"
                target_path = os.path.join(base_dest_path, repo_name)

            final_path = await self._github_service.clone_repo_async(clone_url, target_path)

            if pr_files:
                self.state.pr_target_files = pr_files

            self.state.status_message = tr("store.status.repo_loaded")
            self.state.add_log(f"GitHub Repo Cloned: {final_path}")
            self.state.is_loading = False
            self.state.progress = 0.0
            self.state.notify()

        except Exception as exc:
            self.state.status_message = tr("store.status.clone_error")
            self.state.add_log(f"GitHub Error: {exc}")
            self.state.is_loading = False
            self.state.progress = 0.0
            self.state.notify()
