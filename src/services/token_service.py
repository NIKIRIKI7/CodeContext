"""
Token counting service for the Code Aggregator application.
Uses tiktoken to count tokens for AI models like GPT-4.
"""
import tiktoken
from typing import Callable


class TokenService:
    """Service for counting tokens (uses OpenAI cl100k_base encoding)."""
    
    def __init__(self):
        try:
            self.encoding = tiktoken.get_encoding("cl100k_base")
        except Exception:
            # Fallback, if something goes wrong, though cl100k_base is standard
            self.encoding = tiktoken.encoding_for_model("gpt-4")

    def count(self, text: str) -> int:
        """
        Count the number of tokens in the given text.

        Args:
            text: Text to count tokens for

        Returns:
            Number of tokens in the text
        """
        try:
            return len(self.encoding.encode(text))
        except Exception:
            return 0