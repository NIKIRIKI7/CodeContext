import os
import asyncio
import tempfile
import subprocess
import shutil
import functools


class GitHubService:
    """Сервис для работы с GitHub репозиториями (Async)"""

    @staticmethod
    async def clone_repo_async(url: str) -> str:
        """
        Асинхронно клонирует репозиторий.
        Возвращает путь к временной папке.
        """
        if not url.startswith("http"):
            raise ValueError("Некорректный URL")

        temp_dir = tempfile.mkdtemp(prefix="codecontext_gh_")
        try:
            startupinfo = None
            if os.name == 'nt':
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

            process = await asyncio.create_subprocess_exec(
                "git", "clone", "--depth", "1", url, temp_dir,
                stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.PIPE,
                startupinfo=startupinfo
            )
            _, stderr = await process.communicate()

            if process.returncode != 0:
                err_msg = stderr.decode().strip() if stderr else "Unknown error"
                raise RuntimeError(f"Git clone failed (code {process.returncode}): {err_msg}")
            return temp_dir
        except Exception as e:
            try:
                await asyncio.to_thread(functools.partial(shutil.rmtree, temp_dir, ignore_errors=True))
            except OSError:
                pass
            raise e