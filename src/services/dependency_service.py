import asyncio
from typing import List, Dict, Set, Tuple

# Импортируем стратегии из соседнего файла (точка означает текущую директорию пакета)
from .strategies.dependency_strategies import (
    DependencyParserStrategy,
    PythonDependencyParser,
    WebDependencyParser
)

class DependencyService:
    """
    Сервис для анализа зависимостей.
    Использует паттерн Стратегия для выбора парсера на основе расширения файла.
    """

    def __init__(self):
        # Реестр: { ".py": PythonStrategy(), ".js": WebStrategy(), ... }
        self._parsers: Dict[str, DependencyParserStrategy] = {}
        self._register_defaults()

    def _register_defaults(self):
        """Регистрация стратегий по умолчанию"""
        strategies = [
            PythonDependencyParser(),
            WebDependencyParser()
        ]
        for strategy in strategies:
            self.register_strategy(strategy)

    def register_strategy(self, strategy: DependencyParserStrategy):
        """Метод для динамического добавления новых стратегий"""
        for ext in strategy.supported_extensions:
            self._parsers[ext.lower()] = strategy

    async def resolve_dependencies(self, files: List[Dict[str, str]]) -> Dict[str, Set[str]]:
        """
        Асинхронно анализирует список файлов.
        files: Список словарей {'path': str, 'content': str, 'ext': str}
        """
        if not files:
            return {}

        tasks = [self._process_single_file(file) for file in files]
        results: List[Tuple[str, Set[str]]] = await asyncio.gather(*tasks)

        dependency_map: Dict[str, Set[str]] = {}
        for path, imports in results:
            if imports:
                dependency_map[path] = imports

        return dependency_map

    async def _process_single_file(self, file: Dict[str, str]) -> Tuple[str, Set[str]]:
        """Запуск синхронного анализа в отдельном потоке"""
        return await asyncio.to_thread(self._analyze_sync, file)

    def _analyze_sync(self, file: Dict[str, str]) -> Tuple[str, Set[str]]:
        """Выбор стратегии и парсинг"""
        full_path = file['path']
        content = file['content']

        if not content:
            return full_path, set()

        # Определяем расширение
        ext = file.get('ext')
        if not ext:
            ext = "." + full_path.split('.')[-1].lower() if '.' in full_path else ""

        ext = ext.lower()

        # Получаем стратегию из реестра
        parser = self._parsers.get(ext)

        if parser:
            return full_path, parser.parse(content)

        return full_path, set()