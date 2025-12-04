"""Code cleaning, minification and secret scrubbing."""
import re
from typing import Dict, Any
from src.config import SECRET_PATTERNS

class CodeCleaner:
    # Pre-compiled patterns for performance
    BLOCK_COMMENT_JS = re.compile(r'/\*.*?\*/', re.DOTALL)
    LINE_COMMENT_JS = re.compile(r'//.*$', re.MULTILINE)
    LINE_COMMENT_PY = re.compile(r'#.*$', re.MULTILINE)

    @staticmethod
    def remove_comments(text: str, extension: str) -> str:
        if extension in ['.js', '.ts', '.vue', '.jsx', '.tsx', '.css', '.scss', '.java', '.cpp', '.c', '.h', '.go', '.php']:
            text = CodeCleaner.BLOCK_COMMENT_JS.sub('', text)
            text = CodeCleaner.LINE_COMMENT_JS.sub('', text)
        elif extension in ['.py', '.sh', '.yaml', '.yml', '.rb', '.dockerfile']:
            text = CodeCleaner.LINE_COMMENT_PY.sub('', text)
        return text

    @staticmethod
    def minify_whitespace(text: str) -> str:
        # Fast list comprehension is faster than regex for simple line stripping
        return "\n".join([line.strip() for line in text.splitlines() if line.strip()])

    @staticmethod
    def remove_secrets(text: str) -> str:
        """Masks potential API keys and passwords."""
        for pattern in SECRET_PATTERNS:
            text = pattern.sub(r'\1 [REDACTED]', text)
        return text

    def process(self, text: str, ext: str, options: Dict[str, Any]) -> str:
        if options.get('remove_comments'):
            text = self.remove_comments(text, ext)

        if options.get('remove_secrets'):
            text = self.remove_secrets(text)

        if options.get('minify'):
            text = self.minify_whitespace(text)

        return text