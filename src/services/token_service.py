"""Token service for counting tokens in text for LLM context evaluation."""
import tiktoken


class TokenService:
    """Подсчет токенов для оценки вместимости в контекст LLM."""
    def __init__(self):
        try:
            self.encoding = tiktoken.get_encoding("cl100k_base")  # GPT-4 / GPT-3.5
        except:
            self.encoding = tiktoken.encoding_for_model("gpt-4")

    def count(self, text: str) -> int:
        """
        Count the number of tokens in the given text.
        
        Args:
            text: Input text to count tokens for
            
        Returns:
            Number of tokens in the text
        """
        try:
            return len(self.encoding.encode(text))
        except:
            return 0