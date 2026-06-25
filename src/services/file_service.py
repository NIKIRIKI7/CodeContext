import os
import pathspec
import asyncio
import threading
from pathlib import Path
from typing import List, Callable, Optional, Set

def read_gitignore(folder_path: str) -> List[str]:
    gitignore_path = Path(folder_path) / '.gitignore'
    try:
        return gitignore_path.read_text(encoding='utf-8', errors='replace').splitlines()
    except OSError:
        return []

async def get_git_status_async(repo_path: str) -> dict:
    status_map = {}
    if not Path(repo_path, ".git").exists(): return status_map
    try:
        proc = await asyncio.create_subprocess_exec(
            "git", "-c", "core.quotepath=false", "status", "--porcelain",
            cwd=repo_path, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )
        out, _ = await proc.communicate()
        for line in out.decode('utf-8', errors='replace').splitlines():
            if len(line) > 3:
                state, file_path = line[:2], line[3:].strip().strip('"')
                full_path = str(Path(repo_path) / file_path)
                if 'A' in state or '?' in state: status_map[full_path] = "added"
                elif 'M' in state: status_map[full_path] = "modified"
    except Exception: pass
    return status_map

async def get_git_changed_files_async(repo_path: str, extensions: List[str], ignored: Set[str], git_base: str = "") -> List[str]:
    repo = Path(repo_path)
    if not (repo / ".git").exists(): return []
    try:
        compare_ref = git_base if git_base else "HEAD"
        proc_diff = await asyncio.create_subprocess_exec(
            "git", "-c", "core.quotepath=false", "diff", compare_ref, "--name-only",
            cwd=repo_path, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )
        out_diff, _ = await proc_diff.communicate()
        
        proc_un = await asyncio.create_subprocess_exec(
            "git", "-c", "core.quotepath=false", "ls-files", "--others", "--exclude-standard",
            cwd=repo_path, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )
        out_un, _ = await proc_un.communicate()
        
        all_raw = out_diff.decode('utf-8', errors='replace').splitlines() + out_un.decode('utf-8', errors='replace').splitlines()
        files = set()
        for f in all_raw:
            p = repo / f
            if p.exists() and p.is_file() and any(str(p).endswith(ext) for ext in extensions) and not any(ign in str(p) for ign in ignored):
                files.add(str(p))
        return list(files)
    except Exception:
        return []

class FileService:
    def __init__(self):
        self._watcher_thread = None
        self._watcher_stop = threading.Event()
        self._on_change_callback: Optional[Callable] = None
        
    async def scan_folders_async(self, paths: List[str], extensions_str: str, ignored_str: str, use_git: bool, use_gitignore: bool, git_base: str = "") -> List[str]:
        if not extensions_str.strip():
            from ..utils.config import PRESETS
            extensions_str = PRESETS['Default']['ext']
            
        exts = [e.strip().lower() for e in extensions_str.split()]
        ign = {i.strip() for i in ignored_str.split(',') if i.strip()}
        all_files = []
        
        for path in paths:
            path = os.path.abspath(path)
            if os.path.isfile(path):
                if any(path.lower().endswith(ext) for ext in exts) and not any(i in path for i in ign):
                    all_files.append(path)
            elif os.path.isdir(path):
                if use_git:
                    all_files.extend(await get_git_changed_files_async(path, exts, ign, git_base))
                else:
                    spec = pathspec.PathSpec.from_lines('gitwildmatch', read_gitignore(path)) if use_gitignore else None
                    all_files.extend(await asyncio.to_thread(self._fast_scandir, path, path, ign, exts, spec))
        return sorted(list(set(all_files)))

    def _fast_scandir(self, root_path: str, current_path: str, ign: set, exts: list, spec: pathspec.PathSpec = None) -> List[str]:
        result = []
        try:
            with os.scandir(current_path) as it:
                for entry in it:
                    if entry.name in ign or entry.name == '.git': continue
                    rel_path = os.path.relpath(entry.path, root_path).replace(os.sep, '/')
                    if entry.is_dir(follow_symlinks=False):
                        if not (spec and spec.match_file(rel_path + "/")):
                            result.extend(self._fast_scandir(root_path, entry.path, ign, exts, spec))
                    elif entry.is_file(follow_symlinks=False):
                        if any(entry.name.lower().endswith(ext) for ext in exts) and not (spec and spec.match_file(rel_path)):
                            result.append(entry.path)
        except PermissionError: pass
        return result

    def start_watching(self, paths: List[str], exts_set: Set[str], ign_set: Set[str], on_change: Callable, interval: float = 3.0) -> None:
        self.stop_watching()
        self._on_change_callback = on_change
        self._watcher_stop.clear()
        self._watcher_thread = threading.Thread(target=self._watcher_loop, args=(paths, exts_set, ign_set, interval), daemon=True)
        self._watcher_thread.start()

    def stop_watching(self) -> None:
        self._watcher_stop.set()
        if self._watcher_thread and self._watcher_thread.is_alive():
            self._watcher_thread.join(timeout=2)
            self._watcher_thread = None

    def _watcher_loop(self, paths: List[str], exts_set: Set[str], ign_set: Set[str], interval: float) -> None:
        state: dict = {}
        while not self._watcher_stop.is_set():
            for base in paths:
                for dirpath, dirnames, filenames in os.walk(base):
                    dirnames[:] = [d for d in dirnames if d not in ign_set]
                    for fname in filenames:
                        ext = os.path.splitext(fname)[1].lower()
                        if ext not in exts_set: continue
                        fpath = os.path.join(dirpath, fname)
                        try:
                            mtime = os.path.getmtime(fpath)
                            prev = state.get(fpath)
                            if prev is None: state[fpath] = mtime
                            elif abs(mtime - prev) > 0.5:
                                state[fpath] = mtime
                                if self._on_change_callback: self._on_change_callback()
                        except OSError:
                            state.pop(fpath, None)
            self._watcher_stop.wait(interval)
