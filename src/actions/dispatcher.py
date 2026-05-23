from typing import Any
from ..store.store import Store

class Dispatcher:
    """Диспетчер для отправки действий в Store"""
    def __init__(self, store: Store):
        self.store = store

    def dispatch(self, action_type: str, payload: Any = None):
        """Синхронная отправка действия"""
        self.store.dispatch(action_type, payload)