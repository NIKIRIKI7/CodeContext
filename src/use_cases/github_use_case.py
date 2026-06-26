import os
import re
from ..services.github_service import clone_repo_async, fetch_pr_files_async
from ..store.state import AppState
from src.i18n import tr


async def github_use_case(state: AppState, url: str, base_dest_path: str = "") -> None:
    if not url:
        return

    state.is_loading = True
    state.status_message = tr("github_use_case.preparing")
    state.progress = 0.0
    state.notify()

    pr_files = []
    is_pr = "/pull/" in url
    clone_url = url

    try:
        if is_pr:
            state.add_log(tr("github_use_case.pr_detected"))
            pr_files = await fetch_pr_files_async(url)
            state.add_log(tr("github_use_case.pr_files_count", count=len(pr_files)))
            clone_url = re.sub(r'/pull/\d+.*$', '', url)

        state.status_message = tr("github_use_case.cloning")
        state.progress = 0.3
        state.notify()
        state.add_log(tr("github_use_case.cloning_url", url=clone_url))

        target_path = None
        if base_dest_path:
            repo_name = clone_url.rstrip('/').split('/')[-1]
            if repo_name.endswith('.git'):
                repo_name = repo_name[:-4]
            if not repo_name:
                repo_name = "github_repo"
            target_path = os.path.join(base_dest_path, repo_name)

        final_path = await clone_repo_async(clone_url, target_path)

        if pr_files:
            state.pr_target_files = pr_files

        state.status_message = tr("store.status.repo_loaded")
        state.add_log(f"GitHub Repo Cloned: {final_path}")
        state.is_loading = False
        state.progress = 0.0
        state.notify()

    except Exception as exc:
        state.status_message = tr("store.status.clone_error")
        state.add_log(f"GitHub Error: {exc}")
        state.is_loading = False
        state.progress = 0.0
        state.notify()
