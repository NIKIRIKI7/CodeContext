from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional


@dataclass
class AppSettings:
    """Модель настроек приложения"""
    extensions: str = ".py .js .ts .vue .jsx .tsx .html .css .json .md .sql .xml .yaml .yml .sh .bat .go .java .cpp"
    ignored_paths: str = ".git, node_modules, .nuxt, __pycache__, dist, build, .idea, .vscode, venv, .venv, coverage, .next, target, bin, obj"
    minify: bool = True
    remove_comments: bool = True
    remove_secrets: bool = True
    include_tree: bool = True
    skeleton_mode: bool = False  # <--- NEW FLAG
    use_git: bool = False
    system_prompt: str = "You are an expert software engineer. Analyze the following codebase structure and file contents."
    output_format: str = "markdown"

    # CLI Specifics
    cli_minify: bool = True
    cli_remove_comments: bool = True
    cli_remove_secrets: bool = True
    cli_include_tree: bool = True
    cli_skeleton_mode: bool = False  # <--- NEW FLAG FOR CLI
    cli_format: str = "plain"


@dataclass
class ProcessedFile:
    """Модель обработанного файла"""
    path: str
    content: str
    tokens: int


@dataclass
class AppState:
    """Глобальное состояние приложения"""
    settings: AppSettings = field(default_factory=AppSettings)
    selected_folders: List[str] = field(default_factory=list)
    scanned_files_paths: List[str] = field(default_factory=list)
    processed_files: List[ProcessedFile] = field(default_factory=list)
    final_output_text: str = ""
    total_tokens: int = 0
    status_message: str = "Готов к работе"
    progress: float = 0.0
    is_loading: bool = False
    logs: List[str] = field(default_factory=list)