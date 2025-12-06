import re
from typing import Any
from ..utils.config import SECRET_PATTERNS


class CleanerService:
    """Сервис для очистки и минификации кода"""

    def clean(self, text: str, extension: str, options: Any) -> str:
        """Основной метод очистки"""
        if not text:
            return ""

        if options.remove_comments:
            text = self._remove_comments(text, extension)

        if options.remove_secrets:
            text = self._remove_secrets(text)

        if options.minify:
            text = self._minify_whitespace(text)

        return text

    @staticmethod
    def _remove_comments(text: str, ext: str) -> str:
        if ext in ['.js', '.ts', '.vue', '.jsx', '.tsx', '.css', '.scss', '.java', '.cpp', '.c', '.h', '.go', '.php']:
            # Удаление блочных комментариев /* ... */
            text = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
            # Удаление строчных комментариев // ...
            text = re.sub(r'//.*$', '', text, flags=re.MULTILINE)
        elif ext in ['.py', '.sh', '.yaml', '.yml', '.rb', '.dockerfile', '.toml', '.ini']:
            # Удаление комментариев # ...
            text = re.sub(r'#.*$', '', text, flags=re.MULTILINE)
        return text

    @staticmethod
    def _remove_secrets(text: str) -> str:
        for pattern in SECRET_PATTERNS:
            text = pattern.sub(r'\1 [REDACTED]', text)
        return text

    @staticmethod
    def _minify_whitespace(text: str) -> str:
        return "\n".join([line.strip() for line in text.splitlines() if line.strip()])