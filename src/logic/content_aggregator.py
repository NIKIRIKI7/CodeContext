"""
Content aggregation logic for the Code Aggregator application.
"""
from pathlib import Path
from typing import Generator


class ContentAggregator:
    """
    Responsible for reading files and forming a single text buffer.
    """
    @staticmethod
    def read_file(file_path: Path) -> str:
        """
        Read the content of a file with proper encoding handling.
        
        Args:
            file_path: Path object of the file to read
            
        Returns:
            Content of the file as a string
        """
        try:
            # Try to read in utf-8, fallback if not possible
            return file_path.read_text(encoding='utf-8', errors='replace')
        except Exception as e:
            return f"[ERROR reading file]: {e}"

    def aggregate(self, file_paths: Generator[Path, None, None], logger_callback) -> str:
        """
        Aggregate content from multiple files into a single string.
        
        Args:
            file_paths: Generator of Path objects for files to aggregate
            logger_callback: Function to call with log messages
            
        Returns:
            Aggregated content as a single string
        """
        buffer = []
        count = 0
        logger_callback("[INFO] Starting content aggregation...")
        
        for file_path in file_paths:
            count += 1
            logger_callback(f"Processing: {file_path.name}")
            
            content = self.read_file(file_path)
            separator = f"\n{'='*60}\n"
            header = f"FILE: {file_path}\nDIR: {file_path.parent}\n"
            
            buffer.append(f"{separator}{header}{'-'*60}\n{content}\n")

        logger_callback(f"[SUCCESS] Processed files: {count}")
        return "".join(buffer)