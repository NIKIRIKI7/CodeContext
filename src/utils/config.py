import os
import re
import platform

# Пресеты настроек для разных типов проектов
PRESETS = {
    "Default": {
        "ext": ".py .js .ts .vue .jsx .tsx .html .css .json .md .sql .xml .yaml .yml .sh .bat .go .java .cpp",
        "ign": ".git, node_modules, .nuxt, __pycache__, dist, build, .idea, .vscode, venv, .venv, coverage, .next, target, bin, obj"
    },
    "Python Backend": {
        "ext": ".py .toml .ini .env.example .dockerfile .yaml .sh",
        "ign": "__pycache__, venv, .venv, .git, .idea, .pytest_cache"
    },
    "Web Frontend": {
        "ext": ".js .ts .jsx .tsx .vue .html .css .scss .json",
        "ign": "node_modules, dist, .next, .nuxt, .git, coverage"
    },
    "C++ / Embedded": {
        "ext": ".cpp .c .h .hpp .ino .cmake .txt .xml",
        "ign": "build, bin, obj, .git, .vscode"
    }
}

# Системный промпт по умолчанию
DEFAULT_SYSTEM_PROMPT = "You are an expert software engineer. Analyze the following codebase structure and file contents. Provide improvements, refactoring suggestions, or answer specific questions based on this context."

# Максимальный размер файла (в МБ)
MAX_FILE_SIZE_MB = 2.0

# Паттерны для поиска секретов
SECRET_PATTERNS = [
    re.compile(r'(api[_-]?key|auth[_-]?token|secret[_-]?key|password|pwd)["\']?\s*[:=]\s*["\']([a-zA-Z0-9_\-]{8,})["\']', re.IGNORECASE),
    re.compile(r'(AKIA[0-9A-Z]{16})'),
    re.compile(r'(eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,})')
]

# Путь к шрифту для PDF
def get_font_path():
    system = platform.system()
    if system == "Windows":
        path = os.path.join(os.environ["WINDIR"], "Fonts", "arial.ttf")
        if os.path.exists(path):
            return path
    return None

FONT_PATH = get_font_path()