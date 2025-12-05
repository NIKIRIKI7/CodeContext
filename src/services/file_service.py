import os
from typing import List, Set
from ..data.file_system_repository import FileSystemRepository


class FileService:
    """Сервис для сканирования и сбора путей к файлам"""

    def __init__(self, repo: FileSystemRepository):
        self.repo = repo

    def scan_folders(self, paths: List[str], extensions_str: str, ignored_str: str, use_git: bool) -> List[str]:
        """
        Основной метод сканирования.
        paths: список путей (могут быть папками или файлами).
        """
        # Преобразуем строки настроек в списки/множества
        exts = [e.strip().lower() for e in extensions_str.split()]
        ign = {i.strip() for i in ignored_str.split(',') if i.strip()}

        all_files = []

        for path in paths:
            path = os.path.abspath(path)

            # 1. Если это одиночный файл
            if os.path.isfile(path):
                # Проверяем расширение
                if any(path.lower().endswith(ext) for ext in exts):
                    # Проверяем, не в игноре ли он (по полному пути или имени)
                    if not any(i in path for i in ign):
                        all_files.append(path)
                continue

            # 2. Если это папка - используем репозиторий
            if os.path.isdir(path):
                if use_git:
                    # При Git режиме репозиторий сам разберется с gitignore
                    files = self.repo.get_git_changed_files(path, exts, ign)
                else:
                    files = self.repo.walk_directory(path, ign, exts)
                all_files.extend(files)

        # Удаляем дубликаты и сортируем
        return sorted(list(set(all_files)))