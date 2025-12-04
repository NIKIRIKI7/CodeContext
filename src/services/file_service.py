from typing import List, Set
from ..data.file_system_repository import FileSystemRepository

class FileService:
    """Сервис для сканирования и сбора путей к файлам"""
    
    def __init__(self, repo: FileSystemRepository):
        self.repo = repo

    def scan_folders(self, folders: List[str], extensions_str: str, ignored_str: str, use_git: bool) -> List[str]:
        """Основной метод сканирования"""
        # Подготовка параметров
        exts = [e.strip().lower() for e in extensions_str.split()]
        ign = {i.strip() for i in ignored_str.split(',')}
        
        all_files = []
        
        for folder in folders:
            if use_git:
                files = self.repo.get_git_changed_files(folder, exts, ign)
            else:
                files = self.repo.walk_directory(folder, ign, exts)
            all_files.extend(files)
            
        # Удаляем дубликаты и сортируем
        return sorted(list(set(all_files)))