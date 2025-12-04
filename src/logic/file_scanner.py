"""
File scanning logic for the Code Aggregator application.
"""
import os
from pathlib import Path
from typing import List, Set, Generator
from src.config import IGNORED_DIRS


class FileScanner:
    """
    Responsible for searching files considering filters and ignored directories.
    Uses os.scandir for maximum performance.
    """
    def __init__(self, extensions: List[str], ignored_dirs: Set[str] = None):
        self.extensions = [ext.strip().replace("*", "") for ext in extensions]  # Turn *.py into .py
        self.ignored_dirs = ignored_dirs or IGNORED_DIRS

    def scan(self, root_folders: List[str], logger_callback) -> Generator[Path, None, None]:
        """
        Scan root folders for files matching the specified extensions,
        while ignoring specified directories.
        
        Args:
            root_folders: List of folder paths to scan
            logger_callback: Function to call with log messages
            
        Yields:
            Path objects of files that match the criteria
        """
        for folder in root_folders:
            folder_path = Path(folder)
            if not folder_path.exists():
                logger_callback(f"[WARN] Folder not found: {folder}")
                continue

            yield from self._recursive_scan(folder_path)

    def _recursive_scan(self, path: Path) -> Generator[Path, None, None]:
        """
        Recursively scan a directory and its subdirectories.
        
        Args:
            path: Directory path to scan
            
        Yields:
            Path objects of files that match the criteria
        """
        try:
            # os.scandir is faster than os.walk since it doesn't load all attributes at once
            with os.scandir(path) as it:
                for entry in it:
                    if entry.is_dir():
                        if entry.name in self.ignored_dirs:
                            continue
                        yield from self._recursive_scan(Path(entry.path))
                    elif entry.is_file():
                        if self._is_allowed_extension(entry.name):
                            yield Path(entry.path)
        except PermissionError:
            # Skip system folders without access rights
            pass

    def _is_allowed_extension(self, filename: str) -> bool:
        """
        Check if a file has an allowed extension.
        
        Args:
            filename: Name of the file to check
            
        Returns:
            True if the file extension matches the allowed extensions, False otherwise
        """
        return any(filename.lower().endswith(ext.lower()) for ext in self.extensions)