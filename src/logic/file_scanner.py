"""
File scanning logic for the Code Aggregator application.
"""
import os
import threading
from pathlib import Path
from typing import List, Set, Generator, Callable, Optional
from src.config import DEFAULT_IGNORED, MAX_FILE_SIZE_MB


class FileScanner:
    """
    Responsible for searching files considering filters and ignored directories.
    Uses os.scandir for maximum performance.
    """
    def __init__(self, extensions: List[str], ignored_dirs: Set[str] = None):
        self.extensions = [ext.strip().lower() for ext in extensions]  # Turn *.py into .py
        self.ignored_dirs = ignored_dirs or {x.strip() for x in DEFAULT_IGNORED.split(",")}
        self._stop_event = threading.Event()

    def stop(self):
        """Stop the scanning process."""
        self._stop_event.set()

    def _is_allowed(self, filename: str) -> bool:
        """Check if a file has an allowed extension."""
        return any(filename.lower().endswith(ext) for ext in self.extensions)

    def scan_and_process(self,
                         folders: List[str],
                         progress_callback: Callable[[str, float], None]) -> str:
        """
        Scan folders and process files into XML format with progress tracking.

        Args:
            folders: List of folder paths to scan
            progress_callback: Function to call with progress updates (message, value)

        Returns:
            String with all file contents in XML format
        """
        buffer = []
        file_paths = []

        # 1. Pre-scan to count total files (for progress bar)
        progress_callback("Индексация файлов...", 0.0)
        for folder in folders:
            if not os.path.exists(folder):
                continue
            for root, dirs, files in os.walk(folder):
                if self._stop_event.is_set():
                    return ""
                # Filter directories on the fly
                dirs[:] = [d for d in dirs if d not in self.ignored_dirs]

                for file in files:
                    if self._is_allowed(file):
                        file_paths.append(Path(root) / file)

        total_files = len(file_paths)
        if total_files == 0:
            return ""

        # 2. Read files
        processed = 0
        for path in file_paths:
            if self._stop_event.is_set():
                break

            # Check file size
            try:
                size_mb = path.stat().st_size / (1024 * 1024)
                if size_mb > MAX_FILE_SIZE_MB:
                    progress_callback(f"⚠️ SKIP (Too Big): {path.name}", processed / total_files)
                    continue
            except Exception:
                continue

            try:
                # Read file
                content = path.read_text(encoding='utf-8', errors='replace')

                # Format in XML for LLM context
                entry = (
                    f"<file_context>\n"
                    f"  <path>{path}</path>\n"
                    f"  <content>\n{content}\n  </content>\n"
                    f"</file_context>\n"
                )
                buffer.append(entry)

            except Exception as e:
                print(f"Error reading {path}: {e}")

            processed += 1
            # Update UI every 5 files or at the end
            if processed % 5 == 0 or processed == total_files:
                progress_callback(f"Reading: {path.name}", processed / total_files)

        return "\n".join(buffer)