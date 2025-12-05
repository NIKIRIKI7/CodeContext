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
        Читает содержимое файлов, проверяя ограничения по размеру и бинарности.
        Возвращает список словарей с сырым контентом.
        """
        loaded_files = []

        for path in file_paths:
            try:
                if not os.path.exists(path):
                    continue

                # 1. Проверка размера
                if os.path.getsize(path) > MAX_FILE_SIZE_MB * 1024 * 1024:
                    print(f"Skipping {path}: File too large (> {MAX_FILE_SIZE_MB}MB)")
                    continue

                # 2. Чтение (внутри репозитория теперь есть проверка на бинарность)
                content = self.repo.read_file(path)

                if content is None:
                    # Если вернулся None, значит файл бинарный или произошла ошибка чтения
                    print(f"Skipping {path}: Detected as binary or unreadable")
                    continue

                ext = os.path.splitext(path)[1].lower()
                loaded_files.append({
                    "path": path,
                    "content": content,
                    "ext": ext
                })

            except Exception as e:
                print(f"Error processing {path}: {e}")

        return loaded_files