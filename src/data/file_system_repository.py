import os
import shutil
import stat
import asyncio
from pathlib import Path
from typing import List, Set, Optional, Any


class FileSystemRepository:
    """
    Низкоуровневая работа с файловой системой и Git.
    Использует asyncio.to_thread для блокирующих операций.
    """

    async def read_file_async(self, path: str) -> Optional[str]:
        """Асинхронное чтение файла"""
        return await asyncio.to_thread(self._read_file_sync, path)

    def _read_file_sync(self, path: str) -> Optional[str]:
        if self._is_binary(path):
            return None
        try:
            return Path(path).read_text(encoding='utf-8', errors='replace')
        except (OSError, UnicodeDecodeError):
            return None

    @staticmethod
    def read_gitignore(folder_path: str) -> List[str]:
        gitignore_path = os.path.join(folder_path, '.gitignore')
        if not os.path.exists(gitignore_path):
            return []
        try:
            with open(gitignore_path, 'r', encoding='utf-8') as f:
                return f.readlines()
        except (OSError, UnicodeDecodeError):
            return []

    async def delete_directory_async(self, path: str):
        """Асинхронное удаление директории"""
        await asyncio.to_thread(self._delete_directory_sync, path)

    @staticmethod
    def _delete_directory_sync(path: str):
        if not os.path.exists(path):
            return

        def on_rm_error(func: Any, p: str, exc_info: Any) -> None:
            try:
                os.chmod(p, stat.S_IWRITE)
                os.unlink(p)
            except OSError:
                pass

        try:
            shutil.rmtree(path, onerror=on_rm_error)
        except OSError as e:
            print(f"Error deleting temp dir {path}: {e}")

    @staticmethod
    def _is_binary(path: str) -> bool:
        try:
            with open(path, 'rb') as f:
                chunk = f.read(1024)
                if b'\x00' in chunk:
                    return True
        except OSError:
            pass
        return False

    async def walk_directory_async(self, path: str, ignored_dirs: Set[str], extensions: List[str]) -> List[str]:
        """Асинхронный обход директории (через поток)"""
        return await asyncio.to_thread(self._walk_directory_sync, path, ignored_dirs, extensions)

    @staticmethod
    def _walk_directory_sync(path: str, ignored_dirs: Set[str], extensions: List[str]) -> List[str]:
        result = []
        for root, dirs, files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in ignored_dirs]
            for file in files:
                if any(file.lower().endswith(ext) for ext in extensions):
                    result.append(os.path.join(root, file))
        return result

    @staticmethod
    async def get_git_changed_files_async(repo_path: str, extensions: List[str], ignored_substrings: Set[str]) -> List[
        str]:
        """Асинхронное получение Git changes через asyncio.subprocess"""
        repo = Path(repo_path)
        if not (repo / ".git").exists():
            return []
        try:
            proc_diff = await asyncio.create_subprocess_exec(
                "git", "diff", "HEAD", "--name-only",
                cwd=repo_path,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            out_diff, _ = await proc_diff.communicate()

            proc_untracked = await asyncio.create_subprocess_exec(
                "git", "ls-files", "--others", "--exclude-standard",
                cwd=repo_path,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            out_untracked, _ = await proc_untracked.communicate()

            all_raw = out_diff.decode().splitlines() + out_untracked.decode().splitlines()
            files = set()
            for f in all_raw:
                p = repo / f
                if not p.exists() or p.is_dir():
                    continue
                if not any(str(p).endswith(ext) for ext in extensions):
                    continue
                if any(ign in str(p) for ign in ignored_substrings):
                    continue
                files.add(str(p))
            return list(files)
        except (OSError, ValueError):
            print(f"Git async error in {repo_path}")
            return []

    @staticmethod
    async def get_git_status_async(repo_path: str) -> dict:
        """Асинхронно получает статус файлов из Git (Porcelain)"""
        status_map = {}
        if not Path(repo_path, ".git").exists():
            return status_map
        try:
            proc = await asyncio.create_subprocess_exec(
                "git", "status", "--porcelain",
                cwd=repo_path,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            out, _ = await proc.communicate()
            for line in out.decode('utf-8').splitlines():
                if len(line) > 3:
                    state = line[:2]
                    file_path = line[3:].strip().strip('"')
                    full_path = str(Path(repo_path) / file_path)

                    if 'A' in state or '?' in state:
                        status_map[full_path] = "added"
                    elif 'M' in state:
                        status_map[full_path] = "modified"
        except Exception:
            pass
        return status_map