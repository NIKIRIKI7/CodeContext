import asyncio
import threading
from typing import Coroutine, Any, Optional


class AsyncRuntime:
    """
    Управляет asyncio event loop, запущенным в отдельном потоке.
    Позволяет запускать корутины из синхронного кода (Tkinter).
    """
    _loop: Optional[asyncio.AbstractEventLoop] = None
    _thread: Optional[threading.Thread] = None

    @classmethod
    def start(cls):
        if cls._loop:
            return

        cls._loop = asyncio.new_event_loop()
        asyncio.set_event_loop(cls._loop)

        def run_loop():
            if cls._loop:
                try:
                    cls._loop.run_forever()
                finally:
                    cls._loop.close()

        cls._thread = threading.Thread(target=run_loop, daemon=True, name="AsyncLoopThread")
        cls._thread.start()

    @classmethod
    def run_coroutine(cls, coro: Coroutine[Any, Any, Any]):
        """Отправляет корутину на выполнение в цикл событий"""
        if not cls._loop:
            raise RuntimeError("AsyncRuntime is not started")
        return asyncio.run_coroutine_threadsafe(coro, cls._loop)

    @classmethod
    def stop(cls):
        if cls._loop and cls._loop.is_running():
            def _stop():
                if cls._loop:
                    cls._loop.stop()

            cls._loop.call_soon_threadsafe(_stop)