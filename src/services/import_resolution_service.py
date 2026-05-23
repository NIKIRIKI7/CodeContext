import os
from typing import List
from .strategies.import_strategies import (
    ImportResolutionStrategy,
    StandardImportStrategy,
    FSDImportStrategy,
    AtomicDesignImportStrategy,
    DDDImportStrategy
)


class ImportResolutionService:
    """
    Сервис для разрешения путей импортов.
    Использует паттерн Стратегия для поддержки разных архитектур (FSD, DDD, Atomic).
    """

    def __init__(self):
        self._strategies: List[ImportResolutionStrategy] = []
        self._register_defaults()

    def _register_defaults(self):
        """Регистрация стратегий по умолчанию"""
        self.register_strategy(StandardImportStrategy())
        self.register_strategy(FSDImportStrategy())
        self.register_strategy(AtomicDesignImportStrategy())
        self.register_strategy(DDDImportStrategy())

    def register_strategy(self, strategy: ImportResolutionStrategy):
        """Метод для динамического добавления новых пользовательских стратегий"""
        self._strategies.append(strategy)

    def resolve(self, import_str: str, available_paths: List[str]) -> List[str]:
        """
        Основной метод разрешения путей.
        Находит все файлы в проекте, которые соответствуют строке импорта.
        """
        clean_imp = self._clean_aliases(import_str)

        parts = [p for p in clean_imp.replace('\\', '/').split('/') if p not in ('.', '..', '')]
        if not parts:
            return []

        core_imp = "/".join(parts)
        suffix = "/" + core_imp
        matched = []

        for p in available_paths:
            p_norm = p.replace('\\', '/')
            p_no_ext, _ = os.path.splitext(p_norm)

            # Проверяем путь через все зарегистрированные стратегии
            for strategy in self._strategies:
                if strategy.is_match(p_no_ext, core_imp, suffix, parts):
                    matched.append(p)
                    break  # Если стратегия нашла совпадение, дальше проверять этот файл нет смысла

        return list(set(matched))

    @staticmethod
    def _clean_aliases(import_str: str) -> str:
        """Очистка строки от алиасов и преобразование Python импортов в пути."""
        # Обработка python импортов вида app.domain.models -> app/domain/models
        if '.' in import_str and '/' not in import_str and not import_str.startswith('.'):
            clean_imp = import_str.replace('.', '/')
        else:
            clean_imp = import_str

        # Популярные алиасы frontend-фреймворков
        for prefix in ('@/', '~/', '@', '#'):
            if clean_imp.startswith(prefix):
                return clean_imp[len(prefix):]

        return clean_imp