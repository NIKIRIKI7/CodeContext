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

                spec = None
                if use_gitignore:
                    patterns = self.repo.read_gitignore(path)
                    if patterns:
                        spec = pathspec.PathSpec.from_lines('gitwildmatch', patterns)

                files = await asyncio.to_thread(self._fast_scandir, path, path, ign, exts, spec)
                all_files.extend(files)

        return sorted(list(set(all_files)))

    def _fast_scandir(self, root_path: str, current_path: str, ign: set, exts: list, spec: pathspec.PathSpec = None) -> List[str]:
        result = []
        try:
            with os.scandir(current_path) as it:
                for entry in it:
                    if entry.name in ign or entry.name == '.git':
                        continue
                    rel_path = os.path.relpath(entry.path, root_path)
                    if entry.is_dir(follow_symlinks=False):
                        if spec and spec.match_file(rel_path + "/"):
                            continue
                        result.extend(self._fast_scandir(root_path, entry.path, ign, exts, spec))
                    elif entry.is_file(follow_symlinks=False):
                        if any(entry.name.lower().endswith(ext) for ext in exts):
                            if spec and spec.match_file(rel_path):
                                continue
                            result.append(entry.path)
        except PermissionError:
            pass
        return result

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