"""File scanning service."""
import os
from pathlib import Path
from typing import List, Set
from .git_service import GitService

class FileScanner:
    def __init__(self, extensions: List[str], ignored_dirs: Set[str]):
        self.extensions = [ext.lower() for ext in extensions]
        self.ignored_dirs = ignored_dirs

    def scan(self, root_paths: List[str], use_git: bool = False) -> List[Path]:
        """
        Scan directories. If use_git is True, uses git diff/ls-files.
        """
        all_files = []

        for folder in root_paths:
            if use_git:
                # Try git scan
                git_files = GitService.get_changed_files(folder, self.extensions, self.ignored_dirs)
                if git_files:
                    all_files.extend(git_files)
                    continue
                # If git returned empty (maybe not a repo), fall back or just continue?
                # Let's fall back to normal scan if specific folder isn't a repo is safer,
                # but user asked for CHANGED files. So if not repo, return empty for that folder.
            else:
                # Standard recursive scan
                for root, dirs, files in os.walk(folder):
                    # In-place modification of dirs to prune ignored
                    dirs[:] = [d for d in dirs if d not in self.ignored_dirs]

                    for file in files:
                        if any(file.lower().endswith(ext) for ext in self.extensions):
                            all_files.append(Path(root) / file)

        # Remove duplicates if any
        return list(set(all_files))