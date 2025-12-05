import os
import subprocess
from pathlib import Path
from typing import List, Set, Optional


class FileSystemRepository:
    """Низкоуровневая работа с файловой системой и Git"""

    def read_file(self, path: str) -> Optional[str]:
        """
        Чтение файла с предварительной проверкой на бинарность.
        Возвращает None, если файл бинарный.
        """
        if self._is_binary(path):
            return None

        try:
            return Path(path).read_text(encoding='utf-8', errors='replace')
        except Exception:
            return None

    def _is_binary(self, path: str) -> bool:
        """
        Эвристическая проверка: если в первом блоке (1024 байта)
        есть нулевой байт, считаем файл бинарным.
        """
        try:
            with open(path, 'rb') as f:
                chunk = f.read(1024)
                if b'\x00' in chunk:
                    return True
        except IOError:
            pass
        return False

    def walk_directory(self, path: str, ignored_dirs: Set[str], extensions: List[str]) -> List[str]:
        """Обычное сканирование папки"""
        result = []
        for root, dirs, files in os.walk(path):
            dirs[:] = [d for d in dirs if d not in ignored_dirs]
            for file in files:
                if any(file.lower().endswith(ext) for ext in extensions):
                    result.append(os.path.join(root, file))
        return result

    def get_git_changed_files(self, repo_path: str, extensions: List[str], ignored_substrings: Set[str]) -> List[str]:
        """Получение измененных файлов через Git"""
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
                # Проверка существования, так как файл мог быть удален
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