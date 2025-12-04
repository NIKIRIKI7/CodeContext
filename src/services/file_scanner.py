"""File scanning service to find files efficiently."""
import os
from pathlib import Path
from typing import List, Set, Dict, Any


class FileScanner:
    """Сервис поиска файлов (быстро и асинхронно)."""
    
    def __init__(self, extensions: List[str], ignored_dirs: Set[str]):
        """
        Initialize the file scanner.
        
        Args:
            extensions: List of file extensions to look for
            ignored_dirs: Set of directory names to ignore
        """
        self.extensions = [ext.lower() for ext in extensions]
        self.ignored_dirs = ignored_dirs

    def scan(self, root_paths: List[str]) -> List[Path]:
        """
        Scan directories for files with specified extensions.
        
        Args:
            root_paths: List of root directories to scan
            
        Returns:
            List of file paths found
        """
        files_to_process = []
        
        for folder in root_paths:
            for root, dirs, files in os.walk(folder):
                # Filter out ignored directories
                dirs[:] = [d for d in dirs if d not in self.ignored_dirs]
                
                for file in files:
                    if any(file.lower().endswith(ext) for ext in self.extensions):
                        files_to_process.append(Path(root) / file)
        
        return files_to_process