import os
from typing import List, Dict
from ..data.file_system_repository import FileSystemRepository
from ..utils.config import MAX_FILE_SIZE_MB


class ProcessingService:
    def __init__(self, repo: FileSystemRepository):
        self.repo = repo

    async def read_files_async(self, file_paths: List[str]) -> List[Dict[str, str]]:
        """
        Асинхронно читает файлы параллельно или последовательно.
        """
        loaded_files = []

        # Можно использовать asyncio.gather для параллельного чтения,
        # но для диска часто лучше последовательно или чанками, чтобы не убить IO.
        # Здесь сделаем простой цикл с await.

        for i, path in enumerate(file_paths):
            try:
                if not os.path.exists(path):
                    continue

                # Проверка размера (синхронно, так как stat быстрый)
                if os.path.getsize(path) > MAX_FILE_SIZE_MB * 1024 * 1024:
                    print(f"Skipping {path}: File too large")
                    continue

                content = await self.repo.read_file_async(path)

                if content is None:
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