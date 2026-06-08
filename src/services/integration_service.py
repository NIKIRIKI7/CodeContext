"""
IntegrationService — фасад над ContextMenuStrategy.

Исправлено:
- Убран прямой импорт `winreg` на уровне модуля (падало на Linux/macOS).
- Вся логика делегирована стратегиям из integration_strategies.py (OCP).
- Стратегия выбирается в DIContainer по platform.system() (DIP).
- Класс стал 10-строчным фасадом (SRP).
"""

import platform
from typing import Tuple, Optional

from .strategies.integration_strategies import (
    ContextMenuStrategy,
    WindowsContextMenuStrategy,
    LinuxContextMenuStrategy,
    MacOSContextMenuStrategy,
)


def _create_default_strategy() -> ContextMenuStrategy:
    """Фабрика стратегии по текущей платформе. Используется DIContainer."""
    system = platform.system()
    if system == "Windows":
        return WindowsContextMenuStrategy()
    if system == "Linux":
        return LinuxContextMenuStrategy()
    return MacOSContextMenuStrategy()


class IntegrationService:
    """
    Фасад над ContextMenuStrategy.
    Не знает о реестре Windows или файлах .desktop Linux — только делегирует.
    """

    def __init__(self, strategy: Optional[ContextMenuStrategy] = None):
        # Позволяет подменить стратегию через DI (тестируемость)
        self._strategy = strategy or _create_default_strategy()

    def install_context_menu(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        return self._strategy.install(custom_python_path)

    def remove_context_menu(self) -> Tuple[bool, str]:
        return self._strategy.remove()

    def install_cli(self, custom_python_path: Optional[str] = None) -> Tuple[bool, str]:
        return self._strategy.install_cli(custom_python_path)

    def remove_cli(self) -> Tuple[bool, str]:
        return self._strategy.remove_cli()