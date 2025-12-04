from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional

@dataclass
class AppSettings:
    """Модель настроек приложения"""
    extensions: str = ""
    ignored_paths: str = ""
    minify: bool = True
    remove_comments: bool = True
    remove_secrets: bool = True
    include_tree: bool = True
    use_git: bool = False
    system_prompt: str = ""
    output_format: str = "markdown"

@dataclass
class ProcessedFile:
    """Модель обработанного файла"""
    path: str
    content: str
    tokens: int

@dataclass
class AppState:
    """Глобальное состояние приложения"""
    # Настройки
    settings: AppSettings = field(default_factory=AppSettings)
    
    # Входные данные
    selected_folders: List[str] = field(default_factory=list)
    
    # Результаты работы
    scanned_files_paths: List[str] = field(default_factory=list)
    processed_files: List[ProcessedFile] = field(default_factory=list)
    final_output_text: str = ""
    total_tokens: int = 0
    
    # UI Состояние
    status_message: str = "Готов к работе"
    progress: float = 0.0
    is_loading: bool = False
    logs: List[str] = field(default_factory=list)