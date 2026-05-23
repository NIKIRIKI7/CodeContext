from typing import Any
from ..store.store import Store


class Dispatcher:
    """Тонкая обёртка над Store.dispatch. Передаётся в Use Cases через DI."""

    def __init__(self, store: Store):
        self._store = store

    def dispatch(self, action_type: str, payload: Any = None):
        self._store.dispatch(action_type, payload)