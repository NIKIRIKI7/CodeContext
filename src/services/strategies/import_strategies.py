from abc import ABC, abstractmethod
from typing import List


class ImportResolutionStrategy(ABC):
    """Базовый интерфейс для стратегий разрешения импортов"""

    @abstractmethod
    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        """
        Проверяет, соответствует ли файл запрошенному импорту.
        :param p_no_ext: Нормализованный путь файла в проекте (без расширения)
        :param core_imp: Очищенная от алиасов строка импорта
        :param suffix: Строка импорта с ведущим слэшем (например, '/entities/user')
        :param parts: Строка импорта, разбитая по слэшам
        """
        pass


class StandardImportStrategy(ImportResolutionStrategy):
    """Стратегия для стандартных прямых импортов (например, import { x } from 'entities/user.ts')"""

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        return p_no_ext.endswith(suffix) or p_no_ext == core_imp


class FSDImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для Feature-Sliced Design (FSD).
    Ищет Barrel-файлы вида index.ts или ui/index.vue внутри папки импорта.
    """

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        return p_no_ext.endswith(f"{suffix}/index") or p_no_ext.endswith(f"{suffix}/ui/index")


class AtomicDesignImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для Atomic Design / React паттернов.
    Обрабатывает компоненты, где папка и файл называются одинаково (например, Button/Button.tsx).
    """

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        if not parts:
            return False
        component_name = parts[-1]
        return p_no_ext.endswith(f"{suffix}/{component_name}")


class DDDImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для Domain-Driven Design (DDD) / Python пакетов.
    Ищет точки входа в модули (__init__.py, main.py).
    """

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        return p_no_ext.endswith(f"{suffix}/__init__") or p_no_ext.endswith(f"{suffix}/main")