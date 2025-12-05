import os
import shutil
import stat
import subprocess
from pathlib import Path
from typing import List, Set, Optional


class FileSystemRepository:
    """Низкоуровневая работа с файловой системой и Git"""

    def read_file(self, path: str) -> Optional[str]:
        if self._is_binary(path):
            return None
        try:
            return Path(path).read_text(encoding='utf-8', errors='replace')
        except Exception:
            return None

    def read_gitignore(self, folder_path: str) -> List[str]:
        gitignore_path = os.path.join(folder_path, '.gitignore')
        if not os.path.exists(gitignore_path):
            return []
        try:
            with open(gitignore_path, 'r', encoding='utf-8') as f:
                return f.readlines()
        except Exception:
            return []

    def delete_directory(self, path: str):
        """Рекурсивное удаление папки (для временных файлов)"""
        if not os.path.exists(path):
            return

        def on_rm_error(func, path, exc_info):
            """Обработчик ошибок для удаления read-only файлов (git)"""
            os.chmod(path, stat.S_IWRITE)
            os.unlink(path)

        try:
            shutil.rmtree(path, onerror=on_rm_error)
        except Exception as e:
            print(f"Error deleting temp dir {path}: {e}")

    def _is_binary(self, path: str) -> bool:
        try:
            with open(path, 'rb') as f:
                chunk = f.read(1024)
                if b'\x00' in chunk:
                    return True
        except IOError:
            pass
        return False

    def walk_directory(self, path: str, ignored_dirs: Set[str], extensions: List[str]) -> List[str]:
        result = []
        for root, dirs, files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in ignored_dirs]
            for file in files:
                if any(file.lower().endswith(ext) for ext in extensions):
                    result.append(os.path.join(root, file))
        return result

    def get_git_changed_files(self, repo_path: str, extensions: List[str], ignored_substrings: Set[str]) -> List[str]:
        repo = Path(repo_path)
        if not (repo / ".git").exists():
            return []
        try:
            cmd_diff = ["git", "diff", "HEAD", "--name-only"]
            output_diff = subprocess.check_output(cmd_diff, cwd=repo_path, text=True)
            cmd_untracked = ["git", "ls-files", "--others", "--exclude-standard"]
            output_untracked = subprocess.check_output(cmd_untracked, cwd=repo_path, text=True)

            all_raw = output_diff.splitlines() + output_untracked.splitlines()
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
        except subprocess.CalledProcessError:
            return []