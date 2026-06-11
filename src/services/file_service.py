import os
import pathspec
import asyncio
import time
import threading
from typing import List, Callable, Optional, Set
from ..data.file_system_repository import FileSystemRepository


class FileService:
    def __init__(self, repo: FileSystemRepository):
        self.repo = repo
        self._watcher_thread: Optional[threading.Thread] = None
        self._watcher_stop = threading.Event()
        self._on_change_callback: Optional[Callable] = None

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

    def start_watching(self, paths: List[str], exts_set: Set[str], ign_set: Set[str],
                       on_change: Callable, interval: float = 3.0) -> None:
        """Запускает фоновый поток для отслеживания изменений в файлах (поллинг)"""
        self.stop_watching()
        self._on_change_callback = on_change
        self._watcher_stop.clear()
        self._watcher_thread = threading.Thread(
            target=self._watcher_loop,
            args=(paths, exts_set, ign_set, interval),
            daemon=True
        )
        self._watcher_thread.start()

    def stop_watching(self) -> None:
        self._watcher_stop.set()
        if self._watcher_thread and self._watcher_thread.is_alive():
            self._watcher_thread.join(timeout=2)
        self._watcher_thread = None

    def _watcher_loop(self, paths: List[str], exts_set: Set[str], ign_set: Set[str], interval: float) -> None:
        """Проверяет mtime файлов через интервал и вызывает callback при изменениях"""
        state: dict = {}
        while not self._watcher_stop.is_set():
            changed = False
            for base in paths:
                for dirpath, dirnames, filenames in os.walk(base):
                    dirnames[:] = [d for d in dirnames if d not in ign_set]
                    for fname in filenames:
                        ext = os.path.splitext(fname)[1].lower()
                        if ext not in exts_set:
                            continue
                        fpath = os.path.join(dirpath, fname)
                        try:
                            mtime = os.path.getmtime(fpath)
                            prev = state.get(fpath)
                            if prev is None:
                                state[fpath] = mtime
                                changed = True
                            elif abs(mtime - prev) > 0.5:
                                state[fpath] = mtime
                                changed = True
                        except OSError:
                            state.pop(fpath, None)
                            changed = True
            if changed and self._on_change_callback:
                self._on_change_callback()
            self._watcher_stop.wait(interval)