import os
import json
import asyncio
import tempfile
import subprocess
import shutil
import functools
import urllib.request

from src.i18n import tr
from ..utils.logger import app_logger


class GitHubService:
    """Сервис для работы с GitHub репозиториями (Async)"""

    @staticmethod
    async def clone_repo_async(url: str, dest_path: str = None) -> str:
        """
        Асинхронно клонирует репозиторий.
        Если dest_path указан - клонирует туда. Иначе во временную папку.
        Возвращает путь к папке репозитория.
        """
        if not url.startswith("http"):
            raise ValueError(tr("github_service.invalid_url"))

        is_temp = False
        if not dest_path:
            dest_path = tempfile.mkdtemp(prefix="codecontext_gh_")
            is_temp = True

        try:
            startupinfo = None
            if os.name == 'nt':
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

            process = await asyncio.create_subprocess_exec(
                "git", "clone", "--depth", "1", url, dest_path,
                stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.PIPE,
                startupinfo=startupinfo
            )
            _, stderr = await process.communicate()

            if process.returncode != 0:
                err_msg = stderr.decode().strip() if stderr else "Unknown error"
                raise RuntimeError(f"Git clone failed (code {process.returncode}): {err_msg}")

            return dest_path
        except Exception as e:
            app_logger.error(f"[GitHubService] Clone failed: {e}")
            if is_temp:
                try:
                    await asyncio.to_thread(functools.partial(shutil.rmtree, dest_path, ignore_errors=True))
                except OSError:
                    pass
            raise e

    @staticmethod
    async def fetch_pr_files_async(url: str) -> list:
        parts = url.rstrip('/').split('/')
        if "pull" not in parts:
            return []
        try:
            pr_index = parts.index("pull")
            owner = parts[pr_index - 2]
            repo = parts[pr_index - 1]
            pr_num = parts[pr_index + 1]
            api_url = f"https://api.github.com/repos/{owner}/{repo}/pulls/{pr_num}/files"
            def _fetch():
                req = urllib.request.Request(api_url, headers={"User-Agent": "CodeContextAI"})
                with urllib.request.urlopen(req, timeout=15) as response:
                    data = json.loads(response.read().decode('utf-8'))
                    return [f_obj['filename'] for f_obj in data]
            return await asyncio.to_thread(_fetch)
        except Exception as e:
            app_logger.error(f"[GitHubService] PR files fetch failed: {e}")
            raise RuntimeError(f"GitHub API error: {e}")