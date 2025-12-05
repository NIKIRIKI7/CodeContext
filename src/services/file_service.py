import os
import pathspec
import asyncio
from typing import List
from ..data.file_system_repository import FileSystemRepository


class FileService:
    def __init__(self, repo: FileSystemRepository):
        self.repo = repo

    async def scan_folders_async(self, paths: List[str], extensions_str: str, ignored_str: str, use_git: bool,
                                 use_gitignore: bool) -> List[str]:

        exts = [e.strip().lower() for e in extensions_str.split()]
        ign = {i.strip() for i in ignored_str.split(',') if i.strip()}

        all_files = []

        for path in paths:
            path = os.path.abspath(path)

            if os.path.isfile(path):
                if any(path.lower().endswith(ext) for ext in exts):
                    if not any(i in path for i in ign):
                        all_files.append(path)
                continue

            if os.path.isdir(path):
                if use_git:
                    # Асинхронный git diff
                    files = await self.repo.get_git_changed_files_async(path, exts, ign)
                    all_files.extend(files)
                    continue

                # Асинхронный os.walk
                files = await self.repo.walk_directory_async(path, ign, exts)

                if use_gitignore:
                    # Фильтрация .gitignore может быть тяжелой, обернем ее
                    files = await asyncio.to_thread(self._filter_by_gitignore, path, files)

                all_files.extend(files)

        return sorted(list(set(all_files)))

    def _filter_by_gitignore(self, root_folder: str, files: List[str]) -> List[str]:
        # Логика та же, что и раньше
        patterns = self.repo.read_gitignore(root_folder)
        if not patterns:
            return files
        try:
            spec = pathspec.PathSpec.from_lines('gitwildmatch', patterns)
            kept_files = []
            for file_abs_path in files:
                try:
                    rel_path = os.path.relpath(file_abs_path, root_folder)
                    if not spec.match_file(rel_path):
                        kept_files.append(file_abs_path)
                except ValueError:
                    kept_files.append(file_abs_path)
            return kept_files
        except Exception as e:
            print(f"Error processing .gitignore in {root_folder}: {e}")
            return files