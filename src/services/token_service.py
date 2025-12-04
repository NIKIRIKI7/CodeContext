import tiktoken


class TokenService:
    """Сервис для подсчета токенов (LLM Context)"""

    def __init__(self):
        try:
            self.encoding = tiktoken.get_encoding("cl100k_base")
        except:
            try:
                self.encoding = tiktoken.encoding_for_model("gpt-4")
            except:
                self.encoding = None

    def count_tokens(self, text: str) -> int:
        """Возвращает количество токенов в тексте"""
        if not text:
            return 0

        if self.encoding:
            try:
                return len(self.encoding.encode(text))
            except:
                # Фолбэк на приблизительный подсчет (символы / 4)
                return len(text) // 4

        # Если тиктокена нет
        return len(text) // 4