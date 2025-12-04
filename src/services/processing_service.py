import os
from typing import List, Dict
from ..data.file_system_repository import FileSystemRepository
from ..utils.config import MAX_FILE_SIZE_MB


class ProcessingService:
    """
    Сервис для безопасного чтения и предварительной фильтрации файлов.
    Отвечает только за IO операции и валидацию файлов.
    """

    def __init__(self, repo: FileSystemRepository):
        self.repo = repo

    def read_files(self, file_paths: List[str]) -> List[Dict[str, str]]:
        """
        Читает содержимое файлов, проверяя ограничения по размеру.
        Возвращает список словарей с сырым контентом.
        """
        loaded_files = []

        for path in file_paths:
            try:
                # 1. Проверка существования
                if not os.path.exists(path):
                    continue

                # 2. Проверка размера (защита от переполнения памяти)
                if os.path.getsize(path) > MAX_FILE_SIZE_MB * 1024 * 1024:
                    print(f"Skipping {path}: File too large")
                    continue

                # 3. Чтение через репозиторий
                content = self.repo.read_file(path)

                # Если контент пустой или файл бинарный (репозиторий вернул пустую строку при ошибке)
                if content is None:
                    continue

                ext = os.path.splitext(path)[1].lower()

                loaded_files.append({
                    "path": path,
                    "content": content,
                    "ext": ext
                })

            except Exception as e:
                print(f"Error reading {path}: {e}")

        return loaded_files