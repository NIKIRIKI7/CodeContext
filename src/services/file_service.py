import os
import pathspec
from typing import List, Set
from ..data.file_system_repository import FileSystemRepository


class FileService:
    """Сервис для сканирования и сбора путей к файлам"""

    def __init__(self, repo: FileSystemRepository):
        self.repo = repo

    def scan_folders(self, paths: List[str], extensions_str: str, ignored_str: str, use_git: bool,
                     use_gitignore: bool) -> List[str]:
        """
        Основной метод сканирования.
        paths: список путей (могут быть папками или файлами).
        """
        exts = [e.strip().lower() for e in extensions_str.split()]
        ign = {i.strip() for i in ignored_str.split(',') if i.strip()}

        all_files = []

        for path in paths:
            path = os.path.abspath(path)

            # 1. Если это одиночный файл
            if os.path.isfile(path):
                if any(path.lower().endswith(ext) for ext in exts):
                    if not any(i in path for i in ign):
                        all_files.append(path)
                continue

            # 2. Если это папка
            if os.path.isdir(path):
                # Если включен режим Git, то он сам учитывает gitignore
                if use_git:
                    files = self.repo.get_git_changed_files(path, exts, ign)
                    all_files.extend(files)
                    continue

                # Иначе обычное сканирование (с опциональным pathspec)
                files = self.repo.walk_directory(path, ign, exts)

                if use_gitignore:
                    files = self._filter_by_gitignore(path, files)

                all_files.extend(files)

        return sorted(list(set(all_files)))

    def _filter_by_gitignore(self, root_folder: str, files: List[str]) -> List[str]:
        """
        Фильтрует список файлов, используя .gitignore в корневой папке.
        """
        # Читаем паттерны через репозиторий
        patterns = self.repo.read_gitignore(root_folder)

        if not patterns:
            return files

        try:
            # Создаем спецификацию gitignore
            spec = pathspec.PathSpec.from_lines('gitwildmatch', patterns)

            kept_files = []
            for file_abs_path in files:
                # pathspec работает с относительными путями
                try:
                    rel_path = os.path.relpath(file_abs_path, root_folder)
                    # Если файл НЕ соответствует паттернам игнорирования -> оставляем
                    if not spec.match_file(rel_path):
                        kept_files.append(file_abs_path)
                except ValueError:
                    # Если путь не может быть относительным (разные диски), оставляем
                    kept_files.append(file_abs_path)

            return kept_files

        except Exception as e:
            print(f"Error processing .gitignore in {root_folder}: {e}")
            return files