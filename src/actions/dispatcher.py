from typing import Any
from ..store.store import Store
from ..utils.logger import app_logger


class Dispatcher:
    """Тонкая обёртка над Store.dispatch. Передаётся в Use Cases через DI."""

    def __init__(self, store: Store):
        self._store = store

    def dispatch(self, action_type: str, payload: Any = None):
        # Проверяем, включено ли подробное логирование действий
        if self._store.state.settings.enable_logging:
            try:
                payload_str = str(payload)
                # Обрезаем огромные тексты (например, код файлов), чтобы не засорять лог
                if len(payload_str) > 300:
                    payload_str = payload_str[:300] + "... [TRUNCATED]"

                app_logger.debug(f"DISPATCH ⚡ Action: {action_type} | Payload: {payload_str}")
            except Exception:
                app_logger.debug(f"DISPATCH ⚡ Action: {action_type} | Payload: [Unloggable]")

        self._store.dispatch(action_type, payload)