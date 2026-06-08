import os
import platform
import re

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

PROMPT_PRESETS = {
    "Code Analysis (Default)": "You are an expert software engineer. Analyze the following codebase structure and file contents. Provide improvements, refactoring suggestions, or answer specific questions based on this context.",
    "Code Patcher (JSON)": "You are an advanced AI assistant specialized in code modification. Your goal is to provide code changes in a strict JSON format that a patching script will automatically apply.\n\nSUPPORTED ACTIONS:\n1. \"replace\" - Replaces an exact block of existing code. Requires \"file\", \"search\", \"content\".\n2. \"create\" - Creates a new file. Requires \"file\", \"content\".\n3. \"delete\" - Deletes a file. Requires \"file\".\n4. \"append\" - Adds code to the EOF. Requires \"file\", \"content\".\n5. \"prepend\" - Adds code to the BOF. Requires \"file\", \"content\".\n6. \"insert_before\" - Inserts code before a matched block. Requires \"file\", \"search\", \"content\".\n7. \"insert_after\" - Inserts code after a matched block. Requires \"file\", \"search\", \"content\".\n\nCRITICAL RULES:\n1. Provide enough context in \"search\" to make it uniquely identifiable (3-5 lines).\n2. The patching script is whitespace-insensitive, so don't worry about exact indentation in \"search\", but ensure variable names match.\n3. Provide relative paths in \"file\". Directories will be auto-created.\n4. Return ONLY a valid JSON array.\n\nFORMAT EXAMPLE:\n[\n  {\n    \"action\": \"replace\",\n    \"file\": \"src/main.py\",\n    \"search\": \"def old():\\n    pass\",\n    \"content\": \"def new():\\n    return True\"\n  },\n  {\n    \"action\": \"create\",\n    \"file\": \"tests/test_new.py\",\n    \"content\": \"import pytest\\n\"\n  },\n  {\n    \"action\": \"insert_after\",\n    \"file\": \"src/utils.py\",\n    \"search\": \"import os\",\n    \"content\": \"import sys\"\n  },\n  {\n    \"action\": \"delete\",\n    \"file\": \"deprecated.py\"\n  }\n]",
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

def get_font_path():
    system = platform.system()
    if system == "Windows":
        path = os.path.join(os.environ.get("WINDIR", "C:\\Windows"), "Fonts", "arial.ttf")
        if os.path.exists(path):
            return path
    elif system == "Darwin":
        path = "/Library/Fonts/Arial.ttf"
        if os.path.exists(path):
            return path
    else:
        paths = [
            "/usr/share/fonts/truetype/msttcorefonts/Arial.ttf",
            "/usr/share/fonts/TTF/Arial.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/gnu-free/FreeSans.ttf"
        ]
        for p in paths:
            if os.path.exists(p):
                return p
    return None

FONT_PATH = get_font_path()