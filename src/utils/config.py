import os
import sys
import platform
import re
import json
import urllib.request
import threading

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
    }
}

PROMPT_PRESETS = {
    "Code Analysis (Default)": "You are an expert software engineer. Analyze the following codebase structure and file contents. Provide improvements, refactoring suggestions, or answer specific questions based on this context.",
    "Code Patcher (JSON)": "You are an advanced AI assistant specialized in code modification. Your goal is to provide code changes in a strict JSON format that a patching script will automatically apply.\n\nSUPPORTED ACTIONS:\n1. \"replace\" - Replaces an exact block of existing code. Requires \"file\", \"search\", \"content\".\n2. \"create\" - Creates a new file. Requires \"file\", \"content\".\n3. \"delete\" - Deletes a file. Requires \"file\".\n4. \"append\" - Adds code to the EOF. Requires \"file\", \"content\".\n5. \"prepend\" - Adds code to the BOF. Requires \"file\", \"content\".\n6. \"insert_before\" - Inserts code before a matched block. Requires \"file\", \"search\", \"content\".\n7. \"insert_after\" - Inserts code after a matched block. Requires \"file\", \"search\", \"content\".\n\nCRITICAL RULES:\n1. Provide enough context in \"search\" to make it uniquely identifiable (3-5 lines).\n2. The patching script is whitespace-insensitive, so don't worry about exact indentation in \"search\", but ensure variable names match.\n3. Provide relative paths in \"file\". Directories will be auto-created.\n4. Return ONLY a valid JSON array.\n\nFORMAT EXAMPLE:\n[\n  {\n    \"action\": \"replace\",\n    \"file\": \"src/main.py\",\n    \"search\": \"def old():\\n    pass\",\n    \"content\": \"def new():\\n    return True\"\n  }\n]",
    "Documentation Writer": "You are a technical writer. Based on the provided code, generate comprehensive documentation, including README structure, API references, and installation guides.",
    "Bug Hunter": "You are a QA specialist and security expert. Analyze the provided code for potential bugs, security vulnerabilities, and race conditions. Provide a list of critical issues and how to fix them.",
    "Architecture Review": "You are a software architect. Evaluate the project structure, separation of concerns, and design patterns used. Suggest architectural improvements.",
    "Custom": ""
}

DEFAULT_SYSTEM_PROMPT = PROMPT_PRESETS["Code Analysis (Default)"]
MAX_FILE_SIZE_MB = 2.0

SECRET_PATTERNS = [
    re.compile(r'(api[_-]?key|auth[_-]?token|secret[_-]?key|password|pwd)["\']?\s*[:=]\s*["\']([a-zA-Z0-9_\-]{8,})["\']', re.IGNORECASE),
    re.compile(r'(AKIA[0-9A-Z]{16})'),
    re.compile(r'(eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,})')
]

def get_resource_path(relative_path: str) -> str:
    if getattr(sys, 'frozen', False):
        return os.path.join(sys._MEIPASS, relative_path)
    start = os.path.dirname(os.path.abspath(__file__))
    d = start
    while True:
        candidate = os.path.join(d, relative_path)
        if os.path.exists(candidate):
            return candidate
        parent = os.path.dirname(d)
        if parent == d:
            break
        d = parent
    return os.path.join(start, relative_path)

def get_app_data_dir() -> str:
    system = platform.system()
    if system == "Windows":
        base = os.environ.get("LOCALAPPDATA", os.path.expanduser("~\\AppData\\Local"))
    elif system == "Darwin":
        base = os.path.expanduser("~/Library/Application Support")
    else:
        base = os.environ.get("XDG_CONFIG_HOME", os.path.expanduser("~/.config"))
    app_dir = os.path.join(base, "CodeContextAI")
    os.makedirs(app_dir, exist_ok=True)
    return app_dir

def get_app_version() -> str:
    try:
        ver_path = get_resource_path("VERSION.txt")
        if os.path.exists(ver_path):
            with open(ver_path, "r", encoding="utf-8") as f:
                ver = f.read().strip()
                if ver: return ver
    except Exception:
        pass
    return "1.0.0"

class PricingManager:
    _prices_cache: dict[str, float] = {}
    _is_fetched: bool = False
    _fallback_prices = {
        "gpt-4o-mini": 0.00000015,
        "gpt-4o": 0.0000025,
        "claude-3-5-sonnet": 0.000003,
        "gpt-4-turbo": 0.00001,
        "deepseek-coder": 0.00000014
    }

    @classmethod
    def fetch_prices_sync(cls):
        if cls._is_fetched: return
        try:
            url = "https://openrouter.ai/api/v1/models"
            req = urllib.request.Request(url, headers={"User-Agent": "CodeContextAI-App"})
            with urllib.request.urlopen(req, timeout=5) as response:
                data = json.loads(response.read().decode('utf-8'))
                models = data.get('data', [])
                for m in models:
                    m_id = m.get('id', '').lower()
                    try:
                        price = float(m.get('pricing', {}).get('prompt', 0))
                        cls._prices_cache[m_id] = price
                        short_name = m_id.split('/')[-1]
                        cls._prices_cache[short_name] = price
                    except (ValueError, TypeError):
                        pass
                cls._is_fetched = True
        except Exception:
            pass

    @classmethod
    def fetch_prices_background(cls):
        if cls._is_fetched: return
        thread = threading.Thread(target=cls.fetch_prices_sync, daemon=True)
        thread.start()

    @classmethod
    def get_price(cls, model_name: str) -> float:
        model_lower = model_name.lower()
        if model_lower in cls._prices_cache: return cls._prices_cache[model_lower]
        for k, v in cls._prices_cache.items():
            if k in model_lower or model_lower in k: return v
        if model_lower in cls._fallback_prices: return cls._fallback_prices[model_lower]
        for k, v in cls._fallback_prices.items():
            if k in model_lower or model_lower in k: return v
        return 0.0
