import re
from typing import Any
from ..utils.config import SECRET_PATTERNS


class CleanerService:
    """Сервис для очистки и минификации кода с расширенными опциями"""

    _COMMENT_PATTERNS_BLOCK = re.compile(r'/\*.*?\*/', re.DOTALL)
    _COMMENT_PATTERNS_LINE_JS = re.compile(r'//.*$', re.MULTILINE)
    _COMMENT_PATTERNS_LINE_PY = re.compile(r'#.*$', re.MULTILINE)
    _DOCSTRING_PATTERN = re.compile(r'""".*?"""|\'\'\'.*?\'\'\'', re.DOTALL)
    _IMPORT_PATTERN_PY = re.compile(r'^(import |from )', re.MULTILINE)
    _IMPORT_PATTERN_JS = re.compile(r'^(import |export |require\()', re.MULTILINE)
    _MULTI_BLANK = re.compile(r'\n{3,}')
    _TRAILING_WS = re.compile(r'[ \t]+$', re.MULTILINE)

    def clean(self, text: str, extension: str, options: Any) -> str:
        """Основной метод очистки"""
        if not text:
            return ""

        if options.remove_comments:
            text = self._remove_comments(text, extension)
            if not getattr(options, 'preserve_docstrings', False) and extension == '.py':
                text = self._remove_docstrings(text)
        elif not getattr(options, 'preserve_docstrings', True) and extension == '.py':
            text = self._remove_docstrings(text)

        if options.remove_secrets:
            text = self._remove_secrets(text)

        if not getattr(options, 'preserve_imports', False):
            text = self._remove_imports(text, extension)

        if options.minify:
            text = self._minify_whitespace(text)

        if getattr(options, 'aggressive_minify', False):
            text = self._aggressive_minify(text)

        return text

    @staticmethod
    def _remove_docstrings(text: str) -> str:
        return CleanerService._DOCSTRING_PATTERN.sub('', text)

    @staticmethod
    def _remove_imports(text: str, ext: str) -> str:
        if ext == '.py':
            return CleanerService._IMPORT_PATTERN_PY.sub('', text)
        elif ext in ('.js', '.ts', '.jsx', '.tsx', '.vue'):
            return CleanerService._IMPORT_PATTERN_JS.sub('', text)
        return text

    @staticmethod
    def _remove_comments(text: str, ext: str) -> str:
        if ext in ['.js', '.ts', '.vue', '.jsx', '.tsx', '.css', '.scss', '.java', '.cpp', '.c', '.h', '.go', '.php']:
            text = CleanerService._COMMENT_PATTERNS_BLOCK.sub('', text)
            text = CleanerService._COMMENT_PATTERNS_LINE_JS.sub('', text)
        elif ext in ['.py', '.sh', '.yaml', '.yml', '.rb', '.dockerfile', '.toml', '.ini']:
            text = CleanerService._COMMENT_PATTERNS_LINE_PY.sub('', text)
        return text

    @staticmethod
    def _remove_secrets(text: str) -> str:
        for pattern in SECRET_PATTERNS:
            text = pattern.sub(r'\1 [REDACTED]', text)
        return text

    @staticmethod
    def _minify_whitespace(text: str) -> str:
        return "\n".join([line.strip() for line in text.splitlines() if line.strip()])

    @staticmethod
    def _aggressive_minify(text: str) -> str:
        text = CleanerService._TRAILING_WS.sub('', text)
        text = CleanerService._MULTI_BLANK.sub('\n\n', text)
        return text.strip()
