import tiktoken
from ..utils.logger import app_logger


class TokenService:
    """Сервис для подсчета токенов (LLM Context)"""

    def __init__(self):
        try:
            self.encoding = tiktoken.get_encoding("cl100k_base")
        except Exception:
            app_logger.warning("[Tokens] Failed to load cl100k_base, trying gpt-4")
            try:
                self.encoding = tiktoken.encoding_for_model("gpt-4")
            except Exception:
                app_logger.warning("[Tokens] Failed to load any tiktoken encoding, using fallback")
                self.encoding = None

    def count_tokens(self, text: str) -> int:
        """Возвращает количество токенов в тексте"""
        if not text:
            return 0

        if self.encoding:
            try:
                return len(self.encoding.encode(text))
            except Exception:
                return len(text) // 4

        return len(text) // 4