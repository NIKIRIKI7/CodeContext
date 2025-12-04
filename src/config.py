"""
Configuration and constants for the Code Aggregator application (v3.0).
"""
import platform
import os

# Default file extensions to scan
DEFAULT_EXTENSIONS = ".py .js .ts .vue .jsx .tsx .html .css .json .md .sql .xml .yaml .yml .sh .bat .go .java .cpp"
DEFAULT_IGNORED = ".git, node_modules, .nuxt, __pycache__, dist, build, .idea, .vscode, venv, .venv, coverage, .next, target"

# Constants
MAX_FILE_SIZE_MB = 2.0  # Skip files larger than 2 MB to prevent memory issues

# Font path for PDF generation based on platform
def get_font_path():
    system = platform.system()
    if system == "Windows":
        path = os.path.join(os.environ["WINDIR"], "Fonts", "arial.ttf")
        if os.path.exists(path): return path
    elif system == "Darwin":  # MacOS
        paths = ["/Library/Fonts/Arial.ttf", "/System/Library/Fonts/Helvetica.ttc"]
        for p in paths:
            if os.path.exists(p): return p
    elif system == "Linux":
        paths = ["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                 "/usr/share/fonts/TTF/Arial.ttf"]
        for p in paths:
            if os.path.exists(p): return p
    return None

FONT_PATH = get_font_path()