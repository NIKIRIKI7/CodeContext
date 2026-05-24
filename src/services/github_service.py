import os
import asyncio
import tempfile
import subprocess
import shutil
import functools

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
            raise ValueError("Некорректный URL")

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
            # Удаляем неудачный клон только если папка была временной
            if is_temp:
                try:
                    await asyncio.to_thread(functools.partial(shutil.rmtree, dest_path, ignore_errors=True))
                except OSError:
                    pass
            raise e