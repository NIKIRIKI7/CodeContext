import os
import asyncio
import tempfile
import subprocess


class GitHubService:
    """Сервис для работы с GitHub репозиториями (Async)"""

    async def clone_repo_async(self, url: str) -> str:
        """
        Асинхронно клонирует репозиторий.
        Возвращает путь к временной папке.
        """
        if not url.startswith("http"):
            raise ValueError("Некорректный URL")

        # Создаем папку синхронно (это быстро)
        temp_dir = tempfile.mkdtemp(prefix="codecontext_gh_")

        try:
            # Подготовка параметров для скрытия окна консоли на Windows
            startupinfo = None
            if os.name == 'nt':
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

            # Асинхронный запуск процесса
            process = await asyncio.create_subprocess_exec(
                "git", "clone", "--depth", "1", url, temp_dir,
                stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.PIPE,
                startupinfo=startupinfo
            )

            _, stderr = await process.communicate()

            if process.returncode != 0:
                err_msg = stderr.decode().strip() if stderr else "Unknown error"
                raise Exception(f"Git clone failed (code {process.returncode}): {err_msg}")

            return temp_dir

        except Exception as e:
            # Очистка в случае ошибки
            try:
                await asyncio.to_thread(os.rmdir, temp_dir)
            except:
                pass
            raise e