import os
import subprocess
import tempfile
from typing import Tuple


class GitHubService:
    """Сервис для работы с GitHub репозиториями"""

    def clone_repo(self, url: str) -> Tuple[str, str]:
        """
        Клонирует репозиторий во временную папку.
        Возвращает кортеж (путь_к_папке, имя_папки).
        Выбрасывает исключение при ошибке.
        """
        if not url.startswith("http"):
            raise ValueError("Некорректный URL")

        # Создаем временную директорию
        # Используем mkdtemp, чтобы создать уникальную папку
        temp_dir = tempfile.mkdtemp(prefix="codecontext_gh_")

        try:
            # Запускаем git clone
            # --depth 1 для ускорения (нам не нужна история)
            cmd = ["git", "clone", "--depth", "1", url, temp_dir]

            # Для Windows скрываем окно консоли
            startupinfo = None
            if os.name == 'nt':
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW

            subprocess.check_call(
                cmd,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                startupinfo=startupinfo
            )

            return temp_dir
        except subprocess.CalledProcessError:
            # Если ошибка, удаляем пустую папку
            try:
                os.rmdir(temp_dir)
            except:
                pass
            raise Exception("Ошибка выполнения git clone. Проверьте URL и наличие Git.")
        except Exception as e:
            raise e