import re
from typing import Any
from ..utils.config import SECRET_PATTERNS

_COMMENT_PATTERNS_BLOCK = re.compile(r'/\*.*?\*/', re.DOTALL)
_COMMENT_PATTERNS_LINE_JS = re.compile(r'//.*$', re.MULTILINE)
_COMMENT_PATTERNS_LINE_PY = re.compile(r'#.*$', re.MULTILINE)
_DOCSTRING_PATTERN = re.compile(r'\"\"\"[\s\S]*?\"\"\"|\'\'\'[\s\S]*?\'\'\'', re.DOTALL)
_IMPORT_PATTERN_PY = re.compile(r'^(import |from )', re.MULTILINE)
_IMPORT_PATTERN_JS = re.compile(r'^(import |export |require\()', re.MULTILINE)
_MULTI_BLANK = re.compile(r'\n{3,}')
_TRAILING_WS = re.compile(r'[ \t]+$', re.MULTILINE)

def clean(text: str, extension: str, options: Any) -> str:
    if not text: return ""

    if getattr(options, 'remove_comments', True):
        if extension in ['.js', '.ts', '.vue', '.jsx', '.tsx', '.css', '.scss', '.java', '.cpp', '.c', '.h', '.go', '.php']:
            text = _COMMENT_PATTERNS_BLOCK.sub('', text)
            text = _COMMENT_PATTERNS_LINE_JS.sub('', text)
        elif extension in ['.py', '.sh', '.yaml', '.yml', '.rb', '.dockerfile', '.toml', '.ini']:
            text = _COMMENT_PATTERNS_LINE_PY.sub('', text)

    if not getattr(options, 'preserve_docstrings', False) and extension == '.py':
        text = _DOCSTRING_PATTERN.sub('', text)

    if getattr(options, 'remove_secrets', True):
        for pattern in SECRET_PATTERNS:
            text = pattern.sub(r'\1 [REDACTED]', text)

    if not getattr(options, 'preserve_imports', True):
        if extension == '.py': text = _IMPORT_PATTERN_PY.sub('', text)
        elif extension in ('.js', '.ts', '.jsx', '.tsx', '.vue'): text = _IMPORT_PATTERN_JS.sub('', text)

    if getattr(options, 'minify', True):
        text = "\n".join([line.strip() for line in text.splitlines() if line.strip()])

    if getattr(options, 'aggressive_minify', False):
        text = _TRAILING_WS.sub('', text)
        text = _MULTI_BLANK.sub('\n\n', text)
        text = text.strip()

    return text
