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
        if not extensions_str.strip():
            from ..utils.config import PRESETS
            extensions_str = PRESETS['Default']['ext']
            
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
                    files = await self.repo.get_git_changed_files_async(path, exts, ign)
                    all_files.extend(files)
                    continue

                files = await self.repo.walk_directory_async(path, ign, exts)
                if use_gitignore:
                    files = await asyncio.to_thread(self._filter_by_gitignore, path, files)
                all_files.extend(files)

        return sorted(list(set(all_files)))

    def _filter_by_gitignore(self, root_folder: str, files: List[str]) -> List[str]:
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
                    app_logger.warning(f"[FileService] relpath failed for: {file_abs_path}")
                    kept_files.append(file_abs_path)
            return kept_files
        except Exception as e:
            app_logger.error(f"[FileService] Error processing .gitignore in {root_folder}: {e}")
            return files

    def find_project_root(self, file_path: str) -> str:
        """
        Находит корень проекта, поднимаясь вверх по директориям.
        Ищет индикаторы проектов (Git, Node.js, Python).
        """
        current_dir = os.path.dirname(os.path.abspath(file_path))
        check_dir = current_dir
        for _ in range(7):
            if any(os.path.exists(os.path.join(check_dir, indicator))
                   for indicator in ['.git', 'package.json', 'pyproject.toml', 'requirements.txt']):
                return check_dir
            parent = os.path.dirname(check_dir)
            if parent == check_dir:
                break
            check_dir = parent
        return current_dir