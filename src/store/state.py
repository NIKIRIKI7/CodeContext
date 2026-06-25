import dataclasses
from typing import List, Set, Dict

from PySide6.QtCore import QObject, Signal

from src.i18n import tr


@dataclasses.dataclass
class AppSettings:
    extensions: str = ".py .js .ts .vue .jsx .tsx .html .css .json .md .sql .xml .yaml .yml .sh .bat .go .java .cpp"
    ignored_paths: str = ".git, node_modules, .nuxt, __pycache__, dist, build, .idea, .vscode, venv, .venv, coverage, .next, target, bin, obj"
    minify: bool = True
    remove_comments: bool = True
    remove_secrets: bool = True
    include_tree: bool = True
    include_dependencies: bool = False
    include_mermaid: bool = False
    skeleton_mode: bool = False
    use_git: bool = False
    git_base: str = ""
    use_gitignore: bool = True
    system_prompt: str = "You are an expert software engineer. Analyze the following codebase structure and file contents."
    output_format: str = "markdown"
    template_path: str = ""
    enable_logging: bool = True
    python_interpreter: str = ""
    external_editor: str = ""
    receive_prereleases: bool = False
    llm_check_enabled: bool = False
    llm_api_key: str = ""
    llm_base_url: str = "https://api.openai.com/v1"
    llm_model: str = "gpt-4o-mini"
    recent_workspaces: List[str] = dataclasses.field(default_factory=list)
    visible_tabs: List[str] = dataclasses.field(default_factory=lambda: ["sources", "filters", "prompts", "llm_os", "appearance", "plugins"])
    visible_actions: List[str] = dataclasses.field(default_factory=lambda: ["preview", "clipboard", "chat", "editor", "file"])
    visible_checkboxes: List[str] = dataclasses.field(default_factory=lambda: ["dedup", "aggressive", "checkpoints", "watch"])
    custom_presets: Dict[str, dict] = dataclasses.field(default_factory=dict)
    custom_prompt_presets: Dict[str, str] = dataclasses.field(default_factory=dict)
    language: str = ""
    deduplicate: bool = False
    save_checkpoints: bool = False
    auto_watch: bool = False
    prioritize_entry_files: bool = True
    preserve_docstrings: bool = False
    preserve_imports: bool = False
    aggressive_minify: bool = False
    approved_plugins: List[str] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
class ProcessedFile:
    path: str
    content: str
    tokens: int


class AppState(QObject):
    changed = Signal(object)

    def __init__(self):
        super().__init__()
        self.settings = AppSettings()
        self.selected_folders: List[str] = []
        self.temp_folders: List[str] = []
        self.scanned_files_paths: List[str] = []
        self.scanned_file_metadata: Dict[str, dict] = {}
        self.manual_exclusions: Set[str] = set()
        self.processed_files: List[ProcessedFile] = []
        self.pr_target_files: List[str] = []
        self.final_output_text: str = ""
        self.total_tokens: int = 0
        self.selected_tokens: int = 0
        self.status_message: str = tr("state.status.ready")
        self.progress: float = 0.0
        self.is_loading: bool = False
        self.logs: List[str] = []
        self.toast_message: str = ""
        self.preview_text: str = ""
        self.show_preview: bool = False
        self.preview_history: List[dict] = []
        self.before_after_data: List[dict] = []
        self.show_chat: bool = False
        self.chat_context: str = ""
        self.show_tour: bool = False
        self.tour_steps: List[dict] = []
        self.show_update: bool = False
        self.update_info: dict = {}
        self.show_command_palette: bool = False

    def notify(self):
        self.changed.emit(self)

    def add_log(self, msg: str):
        self.logs.append(str(msg))
        self.notify()
