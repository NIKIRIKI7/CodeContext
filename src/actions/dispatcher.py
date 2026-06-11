import time
from typing import Any
from ..store.store import Store
from ..utils.logger import app_logger


_THROTTLED_ACTIONS = {
    'WORKFLOW_PROGRESS',
    'UI_UPDATE_STATUS',
    'PROCESSING_SUCCESS',
    'FORMATTING_SUCCESS',
}


class Dispatcher:
    """Тонкая обёртка над Store.dispatch с поддержкой Throttling'а для UI-событий."""

    def __init__(self, store: Store):
        self._store = store
        self._last_throttle_time: dict[str, float] = {}

    def dispatch(self, action_type: str, payload: Any = None):
        now = time.time()

        if action_type in _THROTTLED_ACTIONS:
            last = self._last_throttle_time.get(action_type, 0.0)
            progress_val = payload.get('progress', 0) if isinstance(payload, dict) else 0
            if now - last < 0.033 and progress_val < 1.0:
                return
            self._last_throttle_time[action_type] = now

        if self._store.state.settings.enable_logging:
            try:
                payload_str = str(payload)
                if len(payload_str) > 300:
                    payload_str = payload_str[:300] + "... [TRUNCATED]"
                app_logger.debug(f"DISPATCH ⚡ Action: {action_type} | Payload: {payload_str}")
            except Exception:
                app_logger.debug(f"DISPATCH ⚡ Action: {action_type} | Payload: [Unloggable]")

        self._store.dispatch(action_type, payload)
