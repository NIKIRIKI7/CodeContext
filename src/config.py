"""
Configuration and constants for the Code Aggregator application.
"""

# Default file extensions to scan
DEFAULT_EXTENSIONS = [
    "*.vue", "*.scss", "*.ts", "*.json", "*.svg", 
    "*.mjs", "*.kt", "*.xml", "*.html", "*.py", 
    "*.js", "*.css"
]

# Directories to ignore during scanning
IGNORED_DIRS = {
    ".git", "node_modules", ".nuxt", "__pycache__", 
    "dist", "build", ".idea", ".vscode"
}

# Font path for PDF generation on Windows
FONT_PATH_WINDOWS = "C:\\Windows\\Fonts\\arial.ttf"