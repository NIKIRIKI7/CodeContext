import functools
import tiktoken
from ..utils.logger import app_logger

@functools.cache
def _get_encoding():
    try:
        return tiktoken.get_encoding("cl100k_base")
    except Exception:
        app_logger.warning("[Tokens] Failed to load cl100k_base, trying gpt-4")
        try:
            return tiktoken.encoding_for_model("gpt-4")
        except Exception:
            app_logger.warning("[Tokens] Failed to load any tiktoken encoding, using fallback")
            return None

def count_tokens(text: str) -> int:
    if not text: return 0
    encoding = _get_encoding()
    if encoding:
        try:
            return len(encoding.encode(text))
        except Exception:
            return len(text) // 4
    return len(text) // 4
